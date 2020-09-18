//! A module for connecting to voice channels.

mod connection;
mod connection_info;
pub mod constants;
mod error;
pub mod events;
mod handler;
pub mod input;
mod manager;
mod payload;
pub(crate) mod threading;
pub mod tracks;

pub use audiopus::{self as opus, Bitrate};
pub use discortp as packet;
pub use self::{
    error::{DcaError, VoiceError},
    events::{CoreEvent, Event, EventContext, EventHandler, TrackEvent},
    handler::Handler,
    input::{
        ffmpeg,
        ytdl,
    },
    manager::Manager,
    tracks::create_player,
};

use connection_info::ConnectionInfo;
use events::EventData;
use tracks::Track;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub(crate) enum Status {
    Connect(ConnectionInfo),
    Disconnect,
    SetTrack(Option<Track>),
    AddTrack(Track),
    SetBitrate(Bitrate),
    AddEvent(EventData),
    Mute(bool),
    Reconnect,
    RebuildInterconnect,
    Poison,
}
