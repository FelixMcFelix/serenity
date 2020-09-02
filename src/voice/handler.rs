use audiopus::Bitrate;
use crate::{
    constants::VoiceOpCode,
    gateway::InterMessage,
    model::{
        id::{
            ChannelId,
            GuildId,
            UserId
        },
        voice::VoiceState,
    },
    voice::{
        input::Input,
        tracks::{
            Track,
            TrackHandle,
        },
        Event,
        EventContext,
        EventData,
        Status as VoiceStatus,
        connection_info::ConnectionInfo,
        threading,
    },
};
use futures::{
    channel::mpsc::UnboundedSender as Sender,
    sink::SinkExt,
};
use tokio::sync::mpsc::{
    error::SendError,
    self,
    UnboundedSender as TokioSender,
};
use serde_json::json;

/// The handler is responsible for "handling" a single voice connection, acting
/// as a clean API above the inner connection.
///
/// Look into the [`Manager`] for a slightly higher-level interface for managing
/// the existence of handlers.
///
/// **Note**: You should _not_ manually mutate any struct fields. You should
/// _only_ read them. Use methods to mutate them.
///
/// # Examples
///
/// Assuming that you already have a `Manager`, most likely retrieved via a
/// [`Shard`], you can join a guild's voice channel and deafen yourself like so:
///
/// ```rust,ignore
/// // assuming a `manager` has already been bound, hopefully retrieved through
/// // a websocket's connection.
/// use serenity::model::{ChannelId, GuildId};
///
/// let guild_id = GuildId(81384788765712384);
/// let channel_id = ChannelId(85482585546833920);
///
/// let handler = manager.join(Some(guild_id), channel_id);
/// handler.deafen(true);
/// ```
///
/// [`Manager`]: struct.Manager.html
/// [`Shard`]: ../gateway/struct.Shard.html
#[derive(Clone, Debug)]
pub struct Handler {
    /// The ChannelId to be connected to, if any.
    ///
    /// **Note**: This _must not_ be manually mutated. Call [`switch_to`] to
    /// mutate this value.
    ///
    /// [`guild`]: #structfield.guild
    /// [`switch_to`]: #method.switch_to
    pub channel_id: Option<ChannelId>,
    /// The voice server endpoint.
    pub endpoint: Option<String>,
    /// The Id of the guild to be connected to.
    pub guild_id: GuildId,
    /// Whether the current handler is set to deafen voice connections.
    ///
    /// **Note**: This _must not_ be manually mutated. Call [`deafen`] to
    /// mutate this value.
    ///
    /// [`deafen`]: #method.deafen
    pub self_deaf: bool,
    /// Whether the current handler is set to mute voice connections.
    ///
    /// **Note**: This _must not_ be manually mutated. Call [`mute`] to mutate
    /// this value.
    ///
    /// [`mute`]: #method.mute
    pub self_mute: bool,
    /// The internal sender to the voice connection monitor thread.
    sender: TokioSender<VoiceStatus>,
    /// The session Id of the current voice connection, if any.
    ///
    /// **Note**: This _should_ be set through an [`update_state`] call.
    ///
    /// [`update_state`]: #method.update_state
    pub session_id: Option<String>,
    /// The token of the current voice connection, if any.
    ///
    /// **Note**: This _should_ be set through an [`update_server`] call.
    ///
    /// [`update_server`]: #method.update_server
    pub token: Option<String>,
    /// The Id of the current user.
    ///
    /// This is configured via [`new`] or [`standalone`].
    ///
    /// [`new`]: #method.new
    /// [`standalone`]: #method.standalone
    pub user_id: UserId,
    /// Will be set when a `Handler` is made via the [`new`][`Handler::new`]
    /// method.
    ///
    /// When set via [`standalone`][`Handler::standalone`], it will not be
    /// present.
    ws: Option<Sender<InterMessage>>,
}

impl Handler {
    /// Creates a new Handler.
    #[inline]
    pub(crate) fn new(
        guild_id: GuildId,
        ws: Sender<InterMessage>,
        user_id: UserId,
    ) -> Self {
        Self::new_raw(guild_id, Some(ws), user_id)
    }

    /// Creates a new, standalone Handler which is not connected to the primary
    /// WebSocket to the Gateway.
    ///
    /// Actions such as muting, deafening, and switching channels will not
    /// function through this Handler and must be done through some other
    /// method, as the values will only be internally updated.
    ///
    /// For most use cases you do not want this. Only use it if you are using
    /// the voice component standalone from the rest of the library.
    #[inline]
    pub fn standalone(guild_id: GuildId, user_id: UserId) -> Self {
        Self::new_raw(guild_id, None, user_id)
    }

    /// Connects to the voice channel if the following are present:
    ///
    /// - [`endpoint`]
    /// - [`session_id`]
    /// - [`token`]
    ///
    /// If they _are_ all present, then `true` is returned. Otherwise, `false`
    /// is.
    ///
    /// This will automatically be called by [`update_server`] or
    /// [`update_state`] when all three values become present.
    ///
    /// [`endpoint`]: #structfield.endpoint
    /// [`session_id`]: #structfield.session_id
    /// [`token`]: #structfield.token
    /// [`update_server`]: #method.update_server
    /// [`update_state`]: #method.update_state
    pub fn connect(&mut self) -> bool {
        if self.endpoint.is_none() || self.session_id.is_none() || self.token.is_none() {
            return false;
        }

        let endpoint = self.endpoint.clone().unwrap();
        let guild_id = self.guild_id;
        let session_id = self.session_id.clone().unwrap();
        let token = self.token.clone().unwrap();
        let user_id = self.user_id;

        // Safe as all of these being present was already checked.
        self.send(VoiceStatus::Connect(ConnectionInfo {
            endpoint,
            guild_id,
            session_id,
            token,
            user_id,
        }));

        true
    }

    /// Sets whether the current connection to be deafened.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    ///
    /// **Note**: Unlike in the official client, you _can_ be deafened while
    /// not being muted.
    ///
    /// **Note**: If the `Handler` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally deafened.
    ///
    /// [`standalone`]: #method.standalone
    pub fn deafen(&mut self, deaf: bool) {
        self.self_deaf = deaf;

        // Only send an update if there is currently a connected channel.
        //
        // Otherwise, this can be treated as a "settings" update for a
        // connection.
        if self.channel_id.is_some() {
            self.update();
        }
    }

    /// Connect - or switch - to the given voice channel by its Id.
    pub fn join(&mut self, channel_id: ChannelId) {
        self.channel_id = Some(channel_id);

        self.send_join();
    }

    /// Leaves the current voice channel, disconnecting from it.
    ///
    /// This does _not_ forget settings, like whether to be self-deafened or
    /// self-muted.
    ///
    /// **Note**: If the `Handler` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally connected to a
    /// voice channel.
    ///
    /// [`standalone`]: #method.standalone
    pub fn leave(&mut self) {
        // Only send an update if we were in a voice channel.
        if self.channel_id.is_some() {
            self.channel_id = None;
            self.send(VoiceStatus::Disconnect);

            self.update();
        }
    }

    /// Sets whether the current connection is to be muted.
    ///
    /// If there is no live voice connection, then this only acts as a settings
    /// update for future connections.
    ///
    /// **Note**: If the `Handler` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally muted.
    ///
    /// [`standalone`]: #method.standalone
    pub fn mute(&mut self, mute: bool) {
        self.self_mute = mute;

        if self.channel_id.is_some() {
            self.update();
            self.send(VoiceStatus::Mute(mute));
        }
    }

    /// Plays audio from a source, returning a handle for further control.
    ///
    /// This can be a source created via [`voice::ffmpeg`] or [`voice::ytdl`].
    ///
    /// [`voice::ffmpeg`]: input/fn.ffmpeg.html
    /// [`voice::ytdl`]: input/fn.ytdl.html
    pub fn play_source(&mut self, source: Input) -> TrackHandle {
        let (player, handle) = super::create_player(source);
        self.send(VoiceStatus::AddTrack(player));

        handle
    }

    /// Plays audio from a source, returning a handle for further control.
    ///
    /// Unlike [`play_only_source`], this stops all other sources attached
    /// to the channel.
    ///
    /// [`play_only_source`]: #method.play_only_source
    pub fn play_only_source(&mut self, source: Input) -> TrackHandle {
        let (player, handle) = super::create_player(source);
        self.send(VoiceStatus::SetTrack(Some(player)));

        handle
    }

    /// Plays audio from a [`Track`] object.
    ///
    /// This will be one half of the return value of [`voice::create_player`].
    /// The main difference between this function and [`play_source`] is
    /// that this allows for direct manipulation of the [`Track`] object
    /// before it is passed over to the voice and mixing contexts.
    ///
    /// [`voice::create_player`]: tracks/fn.create_player.html
    /// [`Track`]: tracks/struct.Track.html
    /// [`play_source`]: #method.play_source
    pub fn play(&mut self, track: Track) {
        self.send(VoiceStatus::AddTrack(track));
    }

    /// Exclusively plays audio from a [`Track`] object.
    ///
    /// This will be one half of the return value of [`voice::create_player`].
    /// As in [`play_only_source`], this stops all other sources attached to the
    /// channel. Like [`play`], however, this allows for direct manipulation of the
    /// [`Track`] object before it is passed over to the voice and mixing contexts.
    ///
    /// [`voice::create_player`]: tracks/fn.create_player.html
    /// [`Track`]: tracks/struct.Track.html
    /// [`play_only_source`]: #method.play_only_source
    /// [`play`]: #method.play
    pub fn play_only(&mut self, track: Track) {
        self.send(VoiceStatus::SetTrack(Some(track)));
    }

    /// Sets the bitrate for encoding Opus packets sent along
    /// the channel being managed.
    ///
    /// The default rate is 128 kbps.
    /// Sensible values range between `Bits(512)` and `Bits(512_000)`
    /// bits per second.
    /// Alternatively, `Auto` and `Max` remain available.
    pub fn set_bitrate(&mut self, bitrate: Bitrate) {
        self.send(VoiceStatus::SetBitrate(bitrate))
    }

    /// Stops playing audio from all sources, if any are set.
    pub fn stop(&mut self) { self.send(VoiceStatus::SetTrack(None)) }

    /// Attach a global event handler to an audio context. Global events may receive
    /// any [`EventContext`].
    ///
    /// Global timing events will tick regardless of whether audio is playing,
    /// so long as the bot is connected to a voice channel, and have no tracks.
    /// [`TrackEvent`]s will respond to all relevant tracks, giving some audio elements.
    ///
    /// Users **must** ensure that no costly work or blocking occurs
    /// within the supplied function or closure. *Taking excess time could prevent
    /// timely sending of packets, causing audio glitches and delays*.
    ///
    /// [`Track`]: tracks/struct.Track.html
    /// [`TrackEvent`]: events/enum.TrackEvent.html
    /// [`EventContext`]: events/enum.EventContext.html
    pub fn add_global_event<F>(&mut self, event: Event, action: F) 
        where F: FnMut(&EventContext<'_>) -> Option<Event> + Send + Sync + 'static
    {
        self.send(VoiceStatus::AddEvent(EventData::new(event, action)));
    }

    /// Switches the current connected voice channel to the given `channel_id`.
    ///
    /// This has 3 separate behaviors:
    ///
    /// - if the given `channel_id` is equivalent to the current connected
    ///   `channel_id`, then do nothing;
    /// - if the given `channel_id` is _not_ equivalent to the current connected
    ///   `channel_id`, then switch to the given `channel_id`;
    /// - if not currently connected to a voice channel, connect to the given
    ///   one.
    ///
    /// If you are dealing with switching from one group to another, then open
    /// another handler, and optionally drop this one via [`Manager::remove`].
    ///
    /// **Note**: The given `channel_id`, if in a guild, _must_ be in the
    /// current handler's associated guild.
    ///
    /// **Note**: If the `Handler` was created via [`standalone`], then this
    /// will _only_ update whether the connection is internally switched to a
    /// different channel.
    ///
    /// [`Manager::remove`]: struct.Manager.html#method.remove
    /// [`standalone`]: #method.standalone
    pub fn switch_to(&mut self, channel_id: ChannelId) {
        match self.channel_id {
            Some(current_id) if current_id == channel_id => {
                // If already connected to the given channel, do nothing.
            },
            _ => {
                self.channel_id = Some(channel_id);

                self.update();
            },
        }
    }

    /// Updates the voice server data.
    ///
    /// You should only need to use this if you initialized the `Handler` via
    /// [`standalone`].
    ///
    /// Refer to the documentation for [`connect`] for when this will
    /// automatically connect to a voice channel.
    ///
    /// [`connect`]: #method.connect
    /// [`standalone`]: #method.standalone
    pub fn update_server(&mut self, endpoint: &Option<String>, token: &str) {
        self.token = Some(token.to_string());

        if let Some(endpoint) = endpoint.clone() {
            self.endpoint = Some(endpoint);

            if self.session_id.is_some() {
                self.connect();
            }
        } else {
            self.leave();
        }
    }

    /// Updates the internal voice state of the current user.
    ///
    /// You should only need to use this if you initialized the `Handler` via
    /// [`standalone`].
    ///
    /// refer to the documentation for [`connect`] for when this will
    /// automatically connect to a voice channel.
    ///
    /// [`connect`]: #method.connect
    /// [`standalone`]: #method.standalone
    pub fn update_state(&mut self, voice_state: &VoiceState) {
        if self.user_id != voice_state.user_id.0 {
            return;
        }

        self.channel_id = voice_state.channel_id;

        if voice_state.channel_id.is_some() {
            self.session_id = Some(voice_state.session_id.clone());

            if self.endpoint.is_some() && self.token.is_some() {
                self.connect();
            }
        } else {
            self.leave();
        }
    }

    fn new_raw(
        guild_id: GuildId,
        ws: Option<Sender<InterMessage>>,
        user_id: UserId,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        threading::start(guild_id, rx);

        Handler {
            channel_id: None,
            endpoint: None,
            guild_id,
            self_deaf: false,
            self_mute: false,
            sender: tx,
            session_id: None,
            token: None,
            user_id,
            ws,
        }
    }

    /// Sends a message to the thread.
    fn send(&mut self, status: VoiceStatus) {
        // Restart thread if it errored.
        if let Err(SendError(status)) = self.sender.send(status) {
            let (tx, rx) = mpsc::unbounded_channel();

            self.sender = tx;
            self.sender.send(status).unwrap();

            threading::start(self.guild_id, rx);

            self.update();
        }
    }

    fn send_join(&mut self) {
        // Do _not_ try connecting if there is not at least a channel. There
        // does not _necessarily_ need to be a guild.
        if self.channel_id.is_none() {
            return;
        }

        self.update();
    }

    /// Send an update for the current session over WS.
    ///
    /// Does nothing if initialized via [`standalone`].
    ///
    /// [`standalone`]: #method.standalone
    fn update(&mut self) {
        if let Some(ws) = self.ws.as_mut() {
            let map = json!({
                "op": VoiceOpCode::SessionDescription.num(),
                "d": {
                    "channel_id": self.channel_id.map(|c| c.0),
                    "guild_id": self.guild_id.0,
                    "self_deaf": self.self_deaf,
                    "self_mute": self.self_mute,
                }
            });

            let _ = ws.unbounded_send(InterMessage::Json(map));
        }
    }
}

impl Drop for Handler {
    /// Leaves the current connected voice channel, if connected to one, and
    /// forgets all configurations relevant to this Handler.
    fn drop(&mut self) { self.leave(); }
}
