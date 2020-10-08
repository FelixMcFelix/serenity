pub mod error;
pub mod info;

use serenity::{
    model::event::VoiceEvent,
};
use crate::{
    constants::*,
    crypto::Mode as CryptoMode,
    payload,
    tasks::{
        AuxPacketMessage,
        Interconnect,
        MixerConnection,
        MixerMessage,   
        UdpMessage,
    },
    ws::{self, ReceiverExt, SenderExt, WsStream},
};
use discortp::discord::{
    IpDiscoveryPacket,
    IpDiscoveryType,
    MutableIpDiscoveryPacket,
    MutableKeepalivePacket,
};
use error::{Error, Result};
use info::ConnectionInfo;
use tracing::{debug, info};
use serde::Deserialize;
use tokio::{
    time::{timeout_at, Elapsed, Instant},
    net::UdpSocket,
};
use url::Url;
use xsalsa20poly1305::{
    aead::NewAead,
    XSalsa20Poly1305 as Cipher,
};


#[cfg(all(feature = "rustls", not(feature = "native")))]
use ws::create_rustls_client;

#[cfg(feature = "native")]
use ws::create_native_tls_client;

pub(crate) struct Connection {
    pub(crate) connection_info: ConnectionInfo,
}

impl Connection {
    pub(crate) async fn new(mut info: ConnectionInfo, interconnect: &Interconnect) -> Result<Connection> {
        let url = generate_url(&mut info.endpoint)?;

        #[cfg(all(feature = "rustls", not(feature = "native")))]
        let mut client = create_rustls_client(url).await?;

        #[cfg(feature = "native")]
        let mut client = create_native_tls_client(url).await?;

        let mut hello = None;
        let mut ready = None;
        client.send_json(&payload::build_identify(&info)).await?;

        loop {
            let value = match client.recv_json().await? {
                Some(value) => value,
                None => continue,
            };

            match VoiceEvent::deserialize(value)? {
                VoiceEvent::Ready(r) => {
                    ready = Some(r);
                    if hello.is_some(){
                        break;
                    }
                },
                VoiceEvent::Hello(h) => {
                    hello = Some(h);
                    if ready.is_some() {
                        break;
                    }
                },
                other => {
                    debug!("[Voice] Expected ready/hello; got: {:?}", other);

                    return Err(Error::ExpectedHandshake);
                },
            }
        };

        let hello = hello.expect("[Voice] Hello packet expected in connection initialisation, but not found.");
        let ready = ready.expect("[Voice] Ready packet expected in connection initialisation, but not found.");

        if !has_valid_mode(&ready.modes, CryptoMode::Normal) {
            return Err(Error::CryptoModeUnavailable);
        }

        let mut udp = UdpSocket::bind("0.0.0.0:0").await?;
        udp.connect((&ready.ip[..], ready.port)).await?;

        // Follow Discord's IP Discovery procedures, in case NAT tunnelling is needed.
        let mut bytes = [0; IpDiscoveryPacket::const_packet_size()];
        {
            let mut view = MutableIpDiscoveryPacket::new(&mut bytes[..])
                .expect(
                    "[Voice] Too few bytes in 'bytes' for IPDiscovery packet.\
                    (Blame: IpDiscoveryPacket::const_packet_size()?)"
                );
            view.set_pkt_type(IpDiscoveryType::Request);
            view.set_length(70);
            view.set_ssrc(ready.ssrc);
        }

        udp.send(&bytes).await?;

        let (len, _addr) = udp.recv_from(&mut bytes).await?;
        {
            let view = IpDiscoveryPacket::new(&bytes[..len])
                .ok_or_else(|| Error::IllegalDiscoveryResponse)?;

            if view.get_pkt_type() != IpDiscoveryType::Response {
                return Err(Error::IllegalDiscoveryResponse);
            }

            let addr = String::from_utf8_lossy(&view.get_address_raw());
            client.send_json(&payload::build_select_protocol(addr, view.get_port(), CryptoMode::Normal)).await?;
        }

        let cipher = init_cipher(&mut client).await?;

        info!("[Voice] Connected to: {}", info.endpoint);

        info!(
            "[Voice] WS heartbeat duration {}ms.",
            hello.heartbeat_interval,
        );

        let (udp_rx, mut udp_tx) = udp.split();

        let (udp_msg_tx, udp_msg_rx) = flume::unbounded();

        let ssrc = ready.ssrc;
        tokio::spawn(async move {
            info!("[Voice] UDP handle started.");

            let mut keepalive_bytes = [0u8; MutableKeepalivePacket::minimum_packet_size()];
            let mut ka = MutableKeepalivePacket::new(&mut keepalive_bytes[..])
                .expect("[Voice] Insufficient bytes given to keepalive packet.");
            ka.set_ssrc(ssrc);

            let mut ka_time = Instant::now() + UDP_KEEPALIVE_GAP;

            loop {
                use UdpMessage::*;
                match timeout_at(ka_time, udp_msg_rx.recv_async()).await {
                    Err(Elapsed{..}) => {
                        info!("[Voice] Sending UDP Keepalive.");
                        let _ = udp_tx.send(&keepalive_bytes[..]).await;
                        ka_time += UDP_KEEPALIVE_GAP;
                    },
                    Ok(Ok(Packet(p))) => {
                        let _ = udp_tx.send(&p[..]).await;
                    },
                    Ok(Err(_)) | Ok(Ok(Poison)) => {
                        break;
                    },
                }
            }

            info!("[Voice] UDP handle stopped.");
        });

        interconnect.aux_packets.send(AuxPacketMessage::UdpCipher(cipher.clone()))?;
        interconnect.aux_packets.send(AuxPacketMessage::SetKeepalive(hello.heartbeat_interval))?;
        interconnect.aux_packets.send(AuxPacketMessage::SetSsrc(ready.ssrc))?;
        interconnect.aux_packets.send(AuxPacketMessage::Udp(udp_rx))?;
        interconnect.aux_packets.send(AuxPacketMessage::Ws(Box::new(client)))?;

        let mix_conn = MixerConnection{
            cipher,
            udp: udp_msg_tx,
        };

        interconnect.mixer.send(MixerMessage::SetConn(mix_conn, ready.ssrc))?;

        Ok(Connection {
            connection_info: info,
        })
    }

    pub async fn reconnect(&mut self, interconnect: &Interconnect) -> Result<()> {
        let url = generate_url(&mut self.connection_info.endpoint)?;

        // Thread may have died, we want to send to prompt a clean exit
        // (if at all possible) and then proceed as normal.

        #[cfg(all(feature = "rustls", not(feature = "native")))]
        let mut client = create_rustls_client(url).await?;

        #[cfg(feature = "native")]
        let mut client = create_native_tls_client(url).await?;

        client.send_json(&payload::build_resume(&self.connection_info)).await?;

        let mut hello = None;
        let mut resumed = None;

        loop {
            let value = match client.recv_json().await? {
                Some(value) => value,
                None => continue,
            };

            match VoiceEvent::deserialize(value)? {
                VoiceEvent::Resumed => {
                    resumed = Some(());
                    if hello.is_some(){
                        break;
                    }
                },
                VoiceEvent::Hello(h) => {
                    hello = Some(h);
                    if resumed.is_some() {
                        break;
                    }
                },
                other => {
                    debug!("[Voice] Expected resumed/hello; got: {:?}", other);

                    return Err(Error::ExpectedHandshake);
                },
            }
        };

        let hello = hello.expect("[Voice] Hello packet expected in connection initialisation, but not found.");

        interconnect.aux_packets.send(AuxPacketMessage::SetKeepalive(hello.heartbeat_interval))?;
        interconnect.aux_packets.send(AuxPacketMessage::Ws(Box::new(client)))?;

        info!("[Voice] Reconnected to: {}", &self.connection_info.endpoint);
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        info!("[Voice] Disconnected");
    }
}

fn generate_url(endpoint: &mut String) -> Result<Url> {
    if endpoint.ends_with(":80") {
        let len = endpoint.len();

        endpoint.truncate(len - 3);
    }

    Url::parse(&format!("wss://{}/?v={}", endpoint, VOICE_GATEWAY_VERSION))
        .or(Err(Error::EndpointUrl))
}

#[inline]
async fn init_cipher(client: &mut WsStream) -> Result<Cipher> {
    loop {
        let value = match client.recv_json().await? {
            Some(value) => value,
            None => continue,
        };

        match VoiceEvent::deserialize(value)? {
            VoiceEvent::SessionDescription(desc) => {
                // FIXME: check against config.
                if desc.mode != CryptoMode::Normal.to_request_str() {
                    return Err(Error::CryptoModeInvalid);
                }

                return Ok(Cipher::new_varkey(&desc.secret_key)?);
            },
            VoiceEvent::Unknown(op, value) => {
                debug!(
                    "[Voice] Expected ready for key; got: op{}/v{:?}",
                    op.num(),
                    value
                );
            },
            _ => {},
        }
    }
}

#[inline]
fn has_valid_mode<T, It> (modes: It, mode: CryptoMode) -> bool
where T: for<'a> PartialEq<&'a str>,
      It : IntoIterator<Item=T>
{
    modes.into_iter().any(|s| s == mode.to_request_str())
}