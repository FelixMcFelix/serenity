use audiopus::{
    Channels,
    coder::Decoder as OpusDecoder,
    Result as OpusResult,
};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::{
    internal::prelude::*,
    prelude::SerenityError,
};
use parking_lot::Mutex;
use serde_json;
use std::{
    ffi::OsStr,
    fs::File,
    io::{
        self,
        BufRead,
        BufReader,
        ErrorKind as IoErrorKind,
        Read,
        Result as IoResult,
    },
    process::{Child, Command, Stdio},
    result::Result as StdResult,
    sync::Arc,
    time::Duration,
};
use super::{
    AudioSource,
    AudioType,
    DcaError,
    DcaMetadata,
    RawAudioSource,
    VoiceError, 
    audio,
    constants::*,
};
use log::{debug, warn};

struct ChildContainer(Child);

impl Read for ChildContainer {
    fn read(&mut self, buffer: &mut [u8]) -> IoResult<usize> {
        self.0.stdout.as_mut().unwrap().read(buffer)
    }
}

impl Drop for ChildContainer {
    fn drop (&mut self) {
        if let Err(e) = self.0.kill() {
            debug!("[Voice] Error awaiting child process: {:?}", e);
        }
    }
}

struct InputSource<R: Read + Send + 'static> {
    stereo: bool,
    reader: R,
    kind: AudioType,
    decoder: Option<Arc<Mutex<OpusDecoder>>>,
}

impl<R: Read + Send> AudioSource for InputSource<R> {
    fn is_stereo(&mut self) -> bool { self.stereo }

    fn get_type(&self) -> AudioType { self.kind }

    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        // Duplicate this to avoid repeating the stereo check.
        // This should let us unconditionally duplicate samples in the main loop body.
        if true_stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let raw = match self.reader.read_i16::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = f32::from(raw) / 32768.0;

                *float_buffer_element += sample * volume;
            }
        } else {
            let mut float_index = 0;
            for i in 0..float_buffer.len() / 2 {
                let raw = match self.reader.read_i16::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = volume * f32::from(raw) / 32768.0;

                float_buffer[float_index] += sample;
                float_buffer[float_index+1] += sample;

                float_index += 2;
            }
        }

        Some(float_buffer.len())
    }

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        if true_stereo {
            for (i, float_buffer_element) in float_buffer.iter_mut().enumerate() {
                let sample = match self.reader.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };

                *float_buffer_element += sample * volume;
            }
        } else {
            let mut float_index = 0;
            for i in 0..float_buffer.len() / 2 {
                let raw = match self.reader.read_f32::<LittleEndian>() {
                    Ok(v) => v,
                    Err(ref e) => {
                        return if e.kind() == IoErrorKind::UnexpectedEof {
                            Some(i)
                        } else {
                            None
                        }
                    },
                };
                let sample = volume * raw;

                float_buffer[float_index] += sample;
                float_buffer[float_index+1] += sample;

                float_index += 2;
            }
        }

        Some(float_buffer.len())
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        match self.reader.read_i16::<LittleEndian>() {
            Ok(size) => {
                if size <= 0 {
                    warn!("Invalid opus frame size: {}", size);
                    return None;
                }

                let mut frame = Vec::with_capacity(size as usize);

                {
                    let reader = self.reader.by_ref();

                    if reader.take(size as u64).read_to_end(&mut frame).is_err() {
                        return None;
                    }
                }

                Some(frame)
            },
            Err(ref e) => if e.kind() == IoErrorKind::UnexpectedEof {
                Some(Vec::new())
            } else {
                None
            },
        }
    }

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> Option<usize> {
        let decoder_lock = self.decoder.as_mut()?.clone();
        let frame = self.read_opus_frame()?;
        let mut local_buf = [0f32; 960 * 2];

        let count = {
            let mut decoder = decoder_lock.lock();

            decoder.decode_float(frame.as_slice(), &mut local_buf[..], false).ok()?
        };

        for (i, float_buffer_element) in float_buffer.iter_mut().enumerate().take(1920) {
            *float_buffer_element += local_buf[i] * volume;
        }

        Some(count)
    }

    fn is_seekable(&self) -> bool {
        false
    }

    fn seek(&mut self, time: Duration) -> Option<Duration> {
        None
    }

    fn consume(&mut self, amt: usize) -> Option<usize> {
        io::copy(&mut self.by_ref().take(amt as u64), &mut io::sink()).ok().map(|a| a as usize)
    }
}

impl<R: Read + Send> Read for InputSource<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        self.reader.read(buf)
    }
}

impl<R: BufRead + Send> BufRead for InputSource<R> {
    fn fill_buf(&mut self) -> IoResult<&[u8]> {
        self.reader.fill_buf()
    }

    fn consume(&mut self, amt: usize) {
        self.reader.consume(amt)
    }
}

impl<R: Read + Send> RawAudioSource for InputSource<R> {}

/// Opens an audio file through `ffmpeg` and creates an audio source.
pub fn ffmpeg<P: AsRef<OsStr>>(path: P) -> Result<Box<dyn AudioSource + Send>> {
    _ffmpeg(path.as_ref())
}

fn _ffmpeg(path: &OsStr) -> Result<Box<dyn AudioSource + Send>> {
    // Will fail if the path is not to a file on the fs. Likely a YouTube URI.
    let is_stereo = is_stereo(path).unwrap_or(false);
    let stereo_val = if is_stereo { "2" } else { "1" };

    _ffmpeg_optioned(path, &[], &[
        "-f",
        "s16le",
        "-ac",
        stereo_val,
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ], Some(is_stereo))
}

/// Opens an audio file through `ffmpeg` and creates an audio source, with
/// user-specified arguments to pass to ffmpeg.
///
/// Note that this does _not_ build on the arguments passed by the [`ffmpeg`]
/// function.
///
/// # Examples
///
/// Pass options to create a custom ffmpeg streamer:
///
/// ```rust,no_run
/// use serenity::voice;
///
/// let stereo_val = "2";
///
/// let streamer = voice::ffmpeg_optioned("./some_file.mp3", &[], &[
///     "-f",
///     "s16le",
///     "-ac",
///     stereo_val,
///     "-ar",
///     "48000",
///     "-acodec",
///     "pcm_s16le",
///     "-",
/// ]);
///```
pub fn ffmpeg_optioned<P: AsRef<OsStr>>(
    path: P,
    pre_input_args: &[&str],
    args: &[&str],
) -> Result<Box<dyn AudioSource + Send>> {
    _ffmpeg_optioned(path.as_ref(), pre_input_args, args, None)
}

fn _ffmpeg_optioned(
    path: &OsStr,
    pre_input_args: &[&str],
    args: &[&str],
    is_stereo_known: Option<bool>,
) -> Result<Box<dyn AudioSource + Send>> {
    let is_stereo = is_stereo_known
        .or_else(|| is_stereo(path).ok())
        .unwrap_or(false);

    let command = Command::new("ffmpeg")
        .args(pre_input_args)
        .arg("-i")
        .arg(path)
        .args(args)
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(float_pcm(is_stereo, ChildContainer(command)))
}

/// Creates a streamed audio source from a DCA file.
/// Currently only accepts the DCA1 format.
pub fn dca<P: AsRef<OsStr>>(path: P) -> StdResult<Box<dyn AudioSource>, DcaError> {
    _dca(path.as_ref())
}

fn _dca(path: &OsStr) -> StdResult<Box<dyn AudioSource>, DcaError> {
    let file = File::open(path).map_err(DcaError::IoError)?;

    let mut reader = BufReader::new(file);

    let mut header = [0u8; 4];

    // Read in the magic number to verify it's a DCA file.
    reader.read_exact(&mut header).map_err(DcaError::IoError)?;

    if header != b"DCA1"[..] {
        return Err(DcaError::InvalidHeader);
    }

    reader.read_exact(&mut header).map_err(DcaError::IoError)?;

    let size = (&header[..]).read_i32::<LittleEndian>().unwrap();

    // Sanity check
    if size < 2 {
        return Err(DcaError::InvalidSize(size));
    }

    let mut raw_json = Vec::with_capacity(size as usize);

    {
        let json_reader = reader.by_ref();
        json_reader
            .take(size as u64)
            .read_to_end(&mut raw_json)
            .map_err(DcaError::IoError)?;
    }

    let metadata = serde_json::from_slice::<DcaMetadata>(raw_json.as_slice())
        .map_err(DcaError::InvalidMetadata)?;

    Ok(opus(metadata.is_stereo(), reader))
}

/// Creates an Opus audio source. This makes certain assumptions: namely, that the input stream
/// is composed ONLY of opus frames of the variety that Discord expects.
///
/// If you want to decode a `.opus` file, use [`ffmpeg`]
///
/// [`ffmpeg`]: fn.ffmpeg.html
pub fn opus<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource + Send> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Opus,
        decoder: Some(
            Arc::new(Mutex::new(
                // We always want to decode *to* stereo, for mixing reasons.
                OpusDecoder::new(audio::SAMPLE_RATE, Channels::Stereo).unwrap()
            ))
        ),
    })
}

/// Creates a PCM audio source.
pub fn pcm<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource + Send> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::Pcm,
        decoder: None,
    })
}

/// Creates a PCM audio source.
pub fn float_pcm<R: Read + Send + 'static>(is_stereo: bool, reader: R) -> Box<dyn AudioSource + Send> {
    Box::new(InputSource {
        stereo: is_stereo,
        reader,
        kind: AudioType::FloatPcm,
        decoder: None,
    })
}

/// Creates a streamed audio source with `youtube-dl` and `ffmpeg`.
pub fn ytdl(uri: &str) -> Result<Box<dyn AudioSource>> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        uri,
        "-o",
        "-"
    ];

    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(float_pcm(true, BufReader::with_capacity(STEREO_FRAME_SIZE * std::mem::size_of::<f32>() * AUDIO_FRAME_RATE / 2, ChildContainer(ffmpeg))))
}

/// Creates a streamed audio source from YouTube search results with `youtube-dl`,`ffmpeg`, and `ytsearch`.
/// Takes the first video listed from the YouTube search.
pub fn ytdl_search(name: &str) -> Result<Box<dyn AudioSource>> {
    let ytdl_args = [
        "-f",
        "webm[abr>0]/bestaudio/best",
        "-R",
        "infinite",
        "--no-playlist",
        "--ignore-config",
        &format!("ytsearch1:{}",name),
        "-o",
        "-"
    ];

    let ffmpeg_args = [
        "-f",
        "s16le",
        "-ac",
        "2",
        "-ar",
        "48000",
        "-acodec",
        "pcm_f32le",
        "-",
    ];

    let youtube_dl = Command::new("youtube-dl")
        .args(&ytdl_args)
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    let ffmpeg = Command::new("ffmpeg")
        .arg("-i")
        .arg("-")
        .args(&ffmpeg_args)
        .stdin(youtube_dl.stdout.ok_or(SerenityError::Other("Failed to open youtube-dl stdout"))?)
        .stderr(Stdio::null())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(float_pcm(true, BufReader::with_capacity(STEREO_FRAME_SIZE * std::mem::size_of::<f32>() * AUDIO_FRAME_RATE / 2, ChildContainer(ffmpeg))))
}

fn is_stereo(path: &OsStr) -> Result<bool> {
    let args = ["-v", "quiet", "-of", "json", "-show-streams", "-i"];

    let out = Command::new("ffprobe")
        .args(&args)
        .arg(path)
        .stdin(Stdio::null())
        .output()?;

    let value: Value = serde_json::from_reader(&out.stdout[..])?;

    let streams = value
        .as_object()
        .and_then(|m| m.get("streams"))
        .and_then(|v| v.as_array())
        .ok_or(Error::Voice(VoiceError::Streams))?;

    let check = streams.iter().any(|stream| {
        let channels = stream
            .as_object()
            .and_then(|m| m.get("channels").and_then(|v| v.as_i64()));

        channels == Some(2)
    });

    Ok(check)
}


// Shared basis for the below cache-based seekables.
struct AudioCacheCore<T> {
    backing_store: Vec<T>,
    chain: Option<()>,
}

/// A wrapper around an existing [`AudioSource`] which caches
/// the decoded and converted audio data locally in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`RestartableSource`]
/// and [`CompressedSource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with small, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved in **uncompressed floating point** form to minimise the
/// cost of audio processing. This is a significant *3 Mbps (375 kiB/s)*,
/// or 131 MiB of RAM for a 6 minute song.
///
/// [`AudioSource`]: trait.AudioSource.html
/// [`CompressedSource`]: struct.CompressedSource.html
/// [`RestartableSource`]: struct.RestartableSource.html
pub struct MemorySource {
    source: Box<dyn AudioSource>,
}

impl MemorySource {
    pub fn new(source: Box<dyn AudioSource>, length_hint: Option<Duration>) -> Self {
        let store = if let Some(length) = length_hint {
            Vec::with_capacity(std::mem::size_of::<f32>() * STEREO_FRAME_SIZE * (length.as_millis() as usize) / FRAME_LEN_MS)
        } else {
            vec![]
        };
        let a = AudioCacheCore::<f32> {
            backing_store: store,
            chain: None,
        };

        Self {
            source,
        }
    }
}

impl AudioSource for MemorySource {
    fn is_stereo(&mut self) -> bool { self.source.is_stereo() }

    fn get_type(&self) -> AudioType { AudioType::FloatPcm }

    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        unimplemented!()
    }

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        unimplemented!()
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        unimplemented!()
    }

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> Option<usize> {
        unimplemented!()
    }

    fn is_seekable(&self) -> bool {
        true
    }

    fn seek(&mut self, time: Duration) -> Option<Duration> {
        unimplemented!()
    }

    fn consume(&mut self, amt: usize) -> Option<usize> {
        self.source.consume(amt)
    }
}

/// A wrapper around an existing [`AudioSource`] which caches
/// the decoded and converted audio data locally in memory.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`RestartableSource`]
/// and [`MemorySource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with larger, repeatedly used audio
/// tracks shared between sources, and stores the sound data
/// retrieved as **compressed Opus audio**. There is an associated memory cost,
/// but this is far smaller than using a [`MemorySource`].
///
/// [`AudioSource`]: trait.AudioSource.html
/// [`MemorySource`]: struct.MemorySource.html
/// [`RestartableSource`]: struct.RestartableSource.html
pub struct CompressedSource {
    source: Box<dyn AudioSource>,
}

impl CompressedSource {
    pub fn new(source: Box<dyn AudioSource>, length_hint: Option<Duration>) -> Self {
        Self {
            source,
        }
    }
}

impl AudioSource for CompressedSource {
    fn is_stereo(&mut self) -> bool { self.source.is_stereo() }

    fn get_type(&self) -> AudioType { AudioType::FloatPcm }

    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        unimplemented!()
    }

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        unimplemented!()
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        unimplemented!()
    }

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> Option<usize> {
        unimplemented!()
    }

    fn is_seekable(&self) -> bool {
        true
    }

    fn seek(&mut self, time: Duration) -> Option<Duration> {
        unimplemented!()
    }

    fn consume(&mut self, amt: usize) -> Option<usize> {
        self.source.consume(amt)
    }
}

// type Restarter = 

// type LocalUnnamed = Box<dyn AudioSource + RawAudioSource + Send>;
type LocalUnnamed = Box<dyn AudioSource + Send>;

/// A wrapper around a method to create a new [`AudioSource`] which
/// seeks backward by recreating the source.
///
/// The main purpose of this wrapper is to enable seeking on
/// incompatible sources (i.e., ffmpeg output) and to ease resource
/// consumption for commonly reused/shared tracks. [`CompressedSource`]
/// and [`MemorySource`] offer the same functionality with different
/// tradeoffs.
///
/// This is intended for use with single-use audio tracks which
/// may require looping or seeking, but where additional memory
/// cannot be spared. Forward seeks will drain the track until reaching
/// the desired timestamp.
///
/// [`AudioSource`]: trait.AudioSource.html
/// [`MemorySource`]: struct.MemorySource.html
/// [`CompressedSource`]: struct.CompressedSource.html
pub struct RestartableSource {
    position: usize,
    recreator: Box<dyn Fn(Option<Duration>) -> Result<LocalUnnamed> + Send + 'static>,
    source: LocalUnnamed,
}

impl RestartableSource {
    pub fn new(recreator: impl Fn(Option<Duration>) -> Result<LocalUnnamed> + Send + 'static) -> Result<Self> {
        recreator(None)
            .map(move |source| {
                Self {
                    position: 0,
                    recreator: Box::new(recreator),
                    source,
                }
            })
    }

    pub fn ffmpeg<P: AsRef<OsStr> + Send + Clone + Copy + 'static>(path: P) -> Result<Self> {
        Self::new(move |time: Option<Duration>| {
            if let Some(time) = time {

                let is_stereo = is_stereo(path.as_ref()).unwrap_or(false);
                let stereo_val = if is_stereo { "2" } else { "1" };

                let ts = format!("{}.{}", time.as_secs(), time.subsec_millis());
                _ffmpeg_optioned(path.as_ref(), &[
                    "-ss",
                    &ts,
                    ],

                    &[
                    "-f",
                    "s16le",
                    "-ac",
                    stereo_val,
                    "-ar",
                    "48000",
                    "-acodec",
                    "pcm_f32le",
                    "-",
                ], Some(is_stereo))
            } else {
                ffmpeg(path)
            }
        })
    }
}

impl AudioSource for RestartableSource {
    fn is_stereo(&mut self) -> bool { self.source.is_stereo() }

    fn get_type(&self) -> AudioType { self.source.get_type() }

    fn add_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        self.source.add_pcm_frame(float_buffer, true_stereo, volume)
            .map(|sz| { self.position += sz; sz })
    }

    fn add_float_pcm_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], true_stereo: bool, volume: f32) -> Option<usize> {
        self.source.add_float_pcm_frame(float_buffer, true_stereo, volume)
            .map(|sz| { self.position += sz; sz })
    }

    fn read_opus_frame(&mut self) -> Option<Vec<u8>> {
        self.source.read_opus_frame()
    }

    fn decode_and_add_opus_frame(&mut self, float_buffer: &mut [f32; STEREO_FRAME_SIZE], volume: f32) -> Option<usize> {
        self.source.decode_and_add_opus_frame(float_buffer, volume)
            .map(|sz| { self.position += sz; sz })
    }

    fn is_seekable(&self) -> bool {
        true
    }

    fn seek(&mut self, time: Duration) -> Option<Duration> {
        let stereo = self.is_stereo();
        let current_ts = byte_count_to_timestamp(self.position, stereo);

        let future_pos = timestamp_to_byte_count(time, stereo);
        if time < current_ts {
            // FIXME: don't unwrap
            self.source = (self.recreator)(Some(time)).unwrap();
            self.position = future_pos;
        } else {
            if let Some(p) = self.source.consume(future_pos - self.position) {
                self.position += p;
            };
        }

        Some(byte_count_to_timestamp(self.position, stereo))
    }

    fn consume(&mut self, amt: usize) -> Option<usize> {
        self.source.consume(amt)
            .map(|sz| { self.position += sz; sz })
    }
}

fn timestamp_to_byte_count(timestamp: Duration, stereo: bool) -> usize {
    ((timestamp.as_millis() as usize) * (MONO_FRAME_SIZE / FRAME_LEN_MS)) << stereo as usize
}

fn byte_count_to_timestamp(amt: usize, stereo: bool) -> Duration {
    Duration::from_millis((((amt * FRAME_LEN_MS) / MONO_FRAME_SIZE) as u64) >> stereo as u64)
}