use parking_lot::Mutex;
use audiopus::{Bitrate, SampleRate};
use std::{
    sync::{
        mpsc::{
            self,
            Receiver,
            SendError,
            Sender,
            TryRecvError,
        },
        Arc,
    },
    time::Duration,
};
use super::events::{
    Event,
    EventContext,
    EventData,
};

pub const HEADER_LEN: usize = 12;
pub const SAMPLE_RATE: SampleRate = SampleRate::Hz48000;
pub const DEFAULT_BITRATE: Bitrate = Bitrate::BitsPerSecond(128_000);

/// A readable audio source.
pub trait AudioSource: Send {
    fn is_stereo(&mut self) -> bool;

    fn get_type(&self) -> AudioType;

    fn read_pcm_frame(&mut self, buffer: &mut [i16]) -> Option<usize>;

    fn read_opus_frame(&mut self) -> Option<Vec<u8>>;

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; 1920], volume: f32) -> Option<usize>;

    fn is_seekable(&self) -> bool;
}

/// A receiver for incoming audio.
pub trait AudioReceiver: Send {
    fn speaking_update(&mut self, _ssrc: u32, _user_id: u64, _speaking: bool) { }

    #[allow(clippy::too_many_arguments)]
    fn voice_packet(&mut self,
                    _ssrc: u32,
                    _sequence: u16,
                    _timestamp: u32,
                    _stereo: bool,
                    _data: &[i16],
                    _compressed_size: usize) { }

    fn client_connect(&mut self, _ssrc: u32, _user_id: u64) { }

    fn client_disconnect(&mut self, _user_id: u64) { }
}

#[derive(Clone, Copy)]
pub enum AudioType {
    Opus,
    Pcm,
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Control object for audio playback.
///
/// Accessed by both commands and the playback code -- as such, access is
/// always guarded. In particular, you should expect to receive
/// a [`LockedAudio`] when calling [`Handler::play_returning`] or
/// [`Handler::play_only`].
///
/// # Example
///
/// ```rust,ignore
/// use serenity::voice::{Handler, LockedAudio, ffmpeg};
///
/// let handler: Handler = /* ... */;
/// let source = ffmpeg("../audio/my-favourite-song.mp3")?;
/// let safe_audio: LockedAudio = handler.play_only(source);
/// {
///     let audio_lock = safe_audio_control.clone();
///     let mut audio = audio_lock.lock();
///
///     audio.volume(0.5);
/// }
/// ```
///
/// [`LockedAudio`]: type.LockedAudio.html
/// [`Handler::play_only`]: struct.Handler.html#method.play_only
/// [`Handler::play_returning`]: struct.Handler.html#method.play_returning
pub struct Audio {

    /// Whether or not this sound is currently playing.
    ///
    /// Can be controlled with [`play`] or [`pause`] if chaining is desired.
    ///
    /// [`play`]: #method.play
    /// [`pause`]: #method.pause
    pub playing: bool,

    /// The desired volume for playback.
    ///
    /// Sensible values fall between `0.0` and `1.0`.
    ///
    /// Can be controlled with [`volume`] if chaining is desired.
    ///
    /// [`volume`]: #method.volume
    pub volume: f32,

    /// Whether or not the sound has finished, or reached the end of its stream.
    ///
    /// ***Read-only*** for now.
    pub finished: bool,

    /// Underlying data access object.
    ///
    /// *Calling code is not expected to use this.*
    pub source: Box<dyn AudioSource>,

    /// The current position for playback.
    ///
    /// Consider the position fields **read-only** for now.
    pub position: Duration,
    pub position_modified: bool,


    /// List of events attached to this audio track.
    ///
    /// Currently, events are visited by linear scan for eligibility.
    /// This can likely be accelerated.
    pub events: Vec<EventData>,

    /// Channel from which commands are received.
    ///
    /// Audio commands are sent in this manner to ensure that access
    /// occurs in a thread-safe manner, without allowing any external
    /// code to lock access to audio objects and block packet generation.
    pub commands: Receiver<AudioCommand>,
}

impl Audio {
    pub fn new(source: Box<dyn AudioSource>, commands: Receiver<AudioCommand>) -> Self {
        Self {
            playing: true,
            volume: 1.0,
            finished: false,
            source,
            position: Duration::new(0, 0),
            position_modified: false,
            events: vec![],
            commands,
        }
    }

    /// Sets [`playing`] to `true` in a manner that allows method chaining.
    ///
    /// [`playing`]: #structfield.playing
    pub fn play(&mut self) -> &mut Self {
        self.playing = true;

        self
    }

    /// Sets [`playing`] to `false` in a manner that allows method chaining.
    ///
    /// [`playing`]: #structfield.playing
    pub fn pause(&mut self) -> &mut Self {
        self.playing = false;

        self
    }

    pub fn stop(&mut self) -> &mut Self {
        self.finished = true;

        self
    }

    /// Sets [`volume`] in a manner that allows method chaining.
    ///
    /// [`volume`]: #structfield.volume
    pub fn volume(&mut self, volume: f32) -> &mut Self {
        self.volume = volume;

        self
    }

    /// Change the position in the stream for subsequent playback.
    ///
    /// Currently a No-op.
    pub fn position(&mut self, position: Duration) -> &mut Self {
        self.position = position;
        self.position_modified = true;

        self
    }

    /// Steps playback location forward by one frame.
    ///
    /// *Used internally*, although in future this might affect seek position.
    pub(crate) fn step_frame(&mut self) {
        self.position += Duration::from_millis(20);
        self.position_modified = false;
    }

    pub fn process_commands(&mut self) {
        // Note: disconnection and an empty channel are both valid,
        // and should allow the audio object to keep running as intended.
        //
        // However, a paused and disconnected stream MUST be stopped
        // to prevent resource leakage.
        loop {
            match self.commands.try_recv() {
                Ok(cmd) => {
                    use AudioCommand::*;
                    match cmd {
                        Play => {self.play();},
                        Pause => {self.pause();},
                        Stop => {self.stop();},
                        Volume(vol) => {self.volume(vol);},
                        Seek(time) => unimplemented!(),
                        AddEvent(evt) => self.events.push(evt),
                        Do(action) => action(self),
                        Request(tx) => {tx.send(Box::new(AudioState {
                            playing: self.playing,
                            volume: self.volume,
                            finished: self.finished,
                            position: self.position,
                        }));},
                    }
                },
                Err(TryRecvError::Disconnected) => {
                    if !self.playing {
                        self.finished = true;
                    }
                    break;
                },
                Err(TryRecvError::Empty) => {
                    break;
                },
            }
        }
    }

}

#[derive(Debug)]
pub struct AudioState {
    pub playing: bool,
    pub volume: f32,
    pub finished: bool,
    pub position: Duration,
}

/// Threadsafe form of an instance of the [`Audio`] struct, locked behind a
/// Mutex.
///
/// [`Audio`]: struct.Audio.html
pub type LockedAudio = Arc<Mutex<Audio>>;

pub type AudioResult = Result<(), SendError<AudioCommand>>;
pub type AudioQueryResult = Result<Receiver<Box<AudioState>>, SendError<AudioCommand>>;
pub type BlockingAudioQueryResult = Result<Box<AudioState>, SendError<AudioCommand>>;
pub type AudioFn = fn(&mut Audio) -> ();

#[derive(Debug)]
pub struct AudioHandle {
    command_channel: Sender<AudioCommand>,
    seekable: bool,
}

impl AudioHandle {
    pub fn new(command_channel: Sender<AudioCommand>, seekable: bool) -> Self {
        Self {
            command_channel,
            seekable,
        }
    }

    pub fn play(&self) -> AudioResult {
        self.send(AudioCommand::Play)
    }

    pub fn pause(&self) -> AudioResult {
        self.send(AudioCommand::Pause)
    }

    pub fn stop(&self) -> AudioResult {
        self.send(AudioCommand::Stop)
    }

    pub fn set_volume(&self, volume: f32) -> AudioResult {
        self.send(AudioCommand::Volume(volume))
    }

    pub fn seek(&self, position: Duration) -> AudioResult {
        if self.seekable {
            self.send(AudioCommand::Seek(position))
        } else {
            Err(SendError(AudioCommand::Seek(position)))
        }
    }

    pub fn add_event<F>(&self, event: Event, action: F) -> AudioResult 
        where F: FnMut(&mut EventContext<'_>) -> Option<Event> + Send + Sync + 'static
    {
        self.send(AudioCommand::AddEvent(EventData::new(event, action)))
    }

    /// Warn user of taking too much time here...
    pub fn action(&self, action: AudioFn) -> AudioResult {
        self.send(AudioCommand::Do(action))
    }

    pub fn get_info(&self) -> AudioQueryResult {
        let (tx, rx) = mpsc::channel();
        self.send(AudioCommand::Request(tx))
            .map(move |_| rx)
    }

    pub fn get_info_blocking(&self) -> BlockingAudioQueryResult {
        // FIXME: combine into audio error type.
        self.get_info()
            .map(|c| c.recv().unwrap())
    }

    #[inline]
    pub fn send(&self, cmd: AudioCommand) -> AudioResult {
        self.command_channel.send(cmd)
    }
}

pub enum AudioCommand {
    Play,
    Pause,
    Stop,
    Volume(f32),
    Seek(Duration),
    AddEvent(EventData),
    Do(AudioFn),
    Request(Sender<Box<AudioState>>),
}

impl std::fmt::Debug for AudioCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(),std::fmt::Error> {
        use AudioCommand::*;
        write!(f, "AudioCommand::{}", match self {
            Play => "Play".to_string(),
            Pause => "Pause".to_string(),
            Stop => "Stop".to_string(),
            Volume(vol) => format!("Volume({})", vol),
            Seek(d) => format!("Seek({:?})", d),
            AddEvent(evt) => format!("AddEvent({:?})", evt),
            Do(f) => "Do([function])".to_string(),
            Request(tx) => format!("Request({:?})", tx),
        })
    }
}

pub enum PlayMode {
    Play,
    Pause,
    Stop,
}