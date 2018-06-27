use bytes::{Buf, BufMut, Bytes, BytesMut, IntoBuf};
use internal::prelude::*;
use opus::{
    packet as opus_packet,
    Application as CodingMode,
    Bitrate,
    Channels,
    Decoder as OpusDecoder,
    Encoder as OpusEncoder,
};
use sodiumoxide::crypto::secretbox::{
    self,
    Key,
    MACBYTES,
    Nonce,
    NONCEBYTES,
};
use std::{
    collections::HashMap,
    io::{
        Error as IoError,
        ErrorKind as IoErrorKind,
        Write,
    },
};
use super::{
    audio::{
        DEFAULT_BITRATE,
        HEADER_LEN,
        SAMPLE_RATE,
        SILENT_FRAME,
    },
    streamer::{
        SendDecoder,
        SendEncoder,
    },
};
use tokio_codec::{Decoder, Encoder};

pub(crate) struct RxVoicePacket {
    pub is_stereo: bool,
    pub seq: u16,
    pub ssrc: u32,
    pub timestamp: u32,
    pub voice: [i16; 1920],
}

pub(crate) enum TxVoicePacket {
    KeepAlive,
    Audio(Vec<f32>, usize, Bitrate),
    Silence,
}

pub(crate) struct VoiceCodec {
    decoder_map: HashMap<(u32, Channels), SendDecoder>,
    encoder: SendEncoder,
    key: Key,
    sequence: u16,
    ssrc: Bytes,
    timestamp: u32,
}

const AUDIO_POSITION: usize = HEADER_LEN + MACBYTES;

impl VoiceCodec {
    pub(crate) fn new(key: Key, ssrc_in: u32) -> Result<VoiceCodec> {
        let mut encoder = OpusEncoder::new(SAMPLE_RATE, Channels::Stereo, CodingMode::Audio)
            .map(SendEncoder::new)?;

        encoder.set_bitrate(Bitrate::Bits(DEFAULT_BITRATE))?;

        let mut ssrc = BytesMut::with_capacity(4);
        ssrc.put_u32_be(ssrc_in);

        Ok(VoiceCodec {
            decoder_map: HashMap::new(),
            encoder: encoder,
            key,
            sequence: 0,
            ssrc: ssrc.freeze(),
            timestamp: 0,
        })
    }

    fn write_header(&self, buf: &mut BytesMut, audio_packet_length: usize) {
        let total_size = AUDIO_POSITION + audio_packet_length;

        buf.reserve(total_size);

        buf.extend_from_slice(&[0x80, 0x78]);
        buf.put_u16_be(self.sequence);
        buf.put_u32_be(self.timestamp);
        buf.extend_from_slice(&self.ssrc);
        buf.extend_from_slice(&[0u8; 12]);

        // the resize is free, because we already pre alloc'd.
        buf.resize(total_size, 0);
    }

    fn finalize(&mut self, buf: &mut BytesMut) {
        let nonce = Nonce::from_slice(&buf[..NONCEBYTES])
            .expect("[voice] Nonce should be guaranteed from 24-byte slice.");

        // If sodiumoxide 0.1.16 worked on stable, then we could encrypt in place.
        // For now, we have to copy I guess...
        // Unless someone's willing to play with unsafe wizardy.
        let crypt = secretbox::seal(&buf[AUDIO_POSITION..], &nonce, &self.key);
        (&mut buf[HEADER_LEN..]).write(&crypt)
            .expect("[voice] Write of frame into unbounded vec should not fail.");

        self.sequence = self.sequence.wrapping_add(1);
        self.timestamp = self.timestamp.wrapping_add(960);
    }
}

impl Encoder for VoiceCodec {
    type Item = TxVoicePacket;
    type Error = IoError;

    // User will either send a heartbeat or audio of variable length.
    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> StdResult<(), Self::Error> {
        match item {
            TxVoicePacket::KeepAlive => {
                dst.extend_from_slice(&self.ssrc);
            },
            TxVoicePacket::Audio(audio, len, bitrate) => {
                // Reconfigure encoder bitrate.
                // From my testing, it seemed like this needed to be set every cycle.
                if let Err(e) = self.encoder.set_bitrate(bitrate) {
                    warn!("[voice] Bitrate set unsuccessfully: {:?}", e);
                }

                let size = match bitrate {
                    // If user specified, we can calculate.
                    // bits -> bytes, then 20ms means 50fps.
                    Bitrate::Bits(b) => b.abs() / (8 * 50),
                    // Otherwise, just have a *lot* preallocated.
                    _ => 4096,
                } as usize;

                self.write_header(dst, size);

                let _len = self.encoder.encode_float(&audio[..len], &mut dst[AUDIO_POSITION..])
                    .map_err(|_| IoError::new(
                        IoErrorKind::InvalidData,
                        "[voice] Couldn't encode voice data as Opus.")
                    )?;

                self.finalize(dst);
            },
            TxVoicePacket::Silence => {
                self.write_header(dst, SILENT_FRAME.len());

                (&mut dst[AUDIO_POSITION..]).write(&SILENT_FRAME)?;

                self.finalize(dst);
            }
        }
        Ok(())
    }
}

impl Decoder for VoiceCodec {
    type Item = RxVoicePacket;
    type Error = IoError;

    fn decode(&mut self, src: &mut BytesMut) -> StdResult<Option<Self::Item>, Self::Error> {
        let mut buffer = [0i16; 960 * 2];
        let mut header = src.split_to(HEADER_LEN);

        let mut nonce = Nonce([0; NONCEBYTES]);
        nonce.0[..HEADER_LEN].clone_from_slice(&header);

        let mut handle = header.split_off(2).into_buf();
        let seq = handle.get_u16_be();
        let timestamp = handle.get_u32_be();
        let ssrc = handle.get_u32_be();

        secretbox::open(src, &nonce, &self.key)
            .and_then(|mut decrypted| {
                let channels = opus_packet::get_nb_channels(&decrypted)
                    .or(Err(()))?;

                let entry =
                    self.decoder_map.entry((ssrc, channels)).or_insert_with(
                        || OpusDecoder::new(SAMPLE_RATE, channels)
                            .map(SendDecoder::new)
                            .expect("[voice] Decoder construction should never fail.")
                    );

                // Strip RTP Header Extensions (one-byte)
                if decrypted[0] == 0xBE && decrypted[1] == 0xDE {
                    // Read the length bytes as a big-endian u16.
                    let header_extension_len = Bytes::from(&decrypted[2..4])
                        .into_buf()
                        .get_u16_be();

                    let mut offset = 4;
                    for _ in 0..header_extension_len {
                        let byte = decrypted[offset];
                        offset += 1;
                        if byte == 0 {
                            continue;
                        }

                        offset += 1 + (0b1111 & (byte >> 4)) as usize;
                    }

                    while decrypted[offset] == 0 {
                        offset += 1;
                    }

                    decrypted = decrypted.split_off(offset);
                }

                let _len = entry.decode(&decrypted, &mut buffer, false)
                    .or(Err(()))?;

                Ok(RxVoicePacket {
                    is_stereo: channels == Channels::Stereo,
                    seq,
                    ssrc,
                    timestamp,
                    voice: buffer,
                })
            })
            .map(Some)
            .map_err(|_| IoError::new(IoErrorKind::InvalidData, "[voice] Couldn't decode Opus frames."))
    }
}
