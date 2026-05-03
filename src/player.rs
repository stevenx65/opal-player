use std::fs::File;
use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Context;
use rodio::source::Source as RodioSource;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::Result;
use crate::library::TrackInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    Off,
    Playlist,
    Track,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Stopped,
    Playing,
    Paused,
}

pub struct Player {
    _stream: OutputStream,
    _handle: OutputStreamHandle,
    sink: Sink,
    pub current_track: Option<TrackInfo>,
    pub state: PlayState,
    pub volume: f32,
    pub muted: bool,
    pub repeat_mode: RepeatMode,
    pub shuffle: bool,
    pub total_duration: Option<Duration>,
    play_start: Option<Instant>,
    accumulated_elapsed: Duration,
    seeking: bool,
    /// Scrubbing: -1 rewinding, 0 idle, 1 fast-forwarding
    pub scrub_direction: i8,
    pub(crate) scrub_timer: f64,
    pub(crate) scrub_interval: f64,
}

impl Player {
    pub fn new() -> Result<Self> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|e| crate::error::AppError::Audio(e.to_string()))?;

        let sink = Sink::try_new(&handle)
            .map_err(|e| crate::error::AppError::Audio(e.to_string()))?;

        Ok(Self {
            _stream: stream,
            _handle: handle,
            sink,
            current_track: None,
            state: PlayState::Stopped,
            volume: 0.8,
            muted: false,
            repeat_mode: RepeatMode::Off,
            shuffle: false,
            total_duration: None,
            play_start: None,
            accumulated_elapsed: Duration::ZERO,
            seeking: false,
            scrub_direction: 0,
            scrub_timer: 0.0,
            scrub_interval: 0.1,
        })
    }

    pub fn play_file(&mut self, track: &TrackInfo) -> Result<()> {
        self.stop();

        let path = track.path.clone();
        let source = SymphoniaSource::new(&path)?;
        self.total_duration = source.duration.or(track.duration);

        self.sink.append(source);
        self.sink.set_volume(if self.muted { 0.0 } else { self.volume });

        self.current_track = Some(track.clone());
        self.state = PlayState::Playing;
        self.play_start = Some(Instant::now());
        self.accumulated_elapsed = Duration::ZERO;

        Ok(())
    }

    pub fn play_pause(&mut self) {
        match self.state {
            PlayState::Playing => {
                self.sink.pause();
                if let Some(start) = self.play_start.take() {
                    self.accumulated_elapsed += start.elapsed();
                }
                self.state = PlayState::Paused;
            }
            PlayState::Paused => {
                self.sink.play();
                self.play_start = Some(Instant::now());
                self.state = PlayState::Playing;
            }
            PlayState::Stopped => {}
        }
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.state = PlayState::Stopped;
        self.current_track = None;
        self.accumulated_elapsed = Duration::ZERO;
        self.play_start = None;
        self.total_duration = None;
    }

    pub fn seek(&mut self, delta_secs: f64) -> Result<()> {
        if self.current_track.is_none() {
            return Ok(());
        }

        let current_pos = self.get_elapsed();
        let total = self.total_duration.unwrap_or(Duration::from_secs(300));
        let new_pos_secs = (current_pos.as_secs_f64() + delta_secs)
            .max(0.0)
            .min(total.as_secs_f64());

        self.seek_absolute(new_pos_secs)
    }

    pub fn seek_absolute(&mut self, seconds: f64) -> Result<()> {
        let Some(track) = &self.current_track else {
            return Ok(());
        };

        let was_playing = self.state == PlayState::Playing;
        self.seeking = true;

        let path = track.path.clone();
        let source = SymphoniaSource::new_with_seek(&path, seconds)?;

        self.sink.stop();
        self.sink.append(source);
        self.sink.set_volume(if self.muted { 0.0 } else { self.volume });

        self.accumulated_elapsed = Duration::from_secs_f64(seconds);
        self.play_start = if was_playing { Some(Instant::now()) } else { None };

        if !was_playing {
            self.sink.pause();
        }

        self.seeking = false;
        Ok(())
    }

    pub fn change_volume(&mut self, delta: f32) {
        self.volume = (self.volume + delta).clamp(0.0, 1.5);
        if !self.muted {
            self.sink.set_volume(self.volume);
        }
    }

    pub fn set_volume(&mut self, vol: f32) {
        self.volume = vol.clamp(0.0, 1.5);
        if !self.muted {
            self.sink.set_volume(self.volume);
        }
    }

    pub fn toggle_mute(&mut self) {
        self.muted = !self.muted;
        if self.muted {
            self.sink.set_volume(0.0);
        } else {
            self.sink.set_volume(self.volume);
        }
    }

    pub fn get_elapsed(&self) -> Duration {
        let running = self.play_start.map(|s| s.elapsed()).unwrap_or(Duration::ZERO);
        self.accumulated_elapsed + running
    }

    /// Called every frame to advance scrubbing. Returns the seek delta in
    /// seconds if enough time has accumulated, or None.
    pub fn scrub_tick(&mut self, dt: f64) -> Option<f64> {
        if self.scrub_direction == 0 {
            return None;
        }
        self.scrub_timer += dt;
        if self.scrub_timer >= self.scrub_interval {
            let steps = (self.scrub_timer / self.scrub_interval) as u32;
            self.scrub_timer -= steps as f64 * self.scrub_interval;
            // 2 seconds of seek per 100ms interval = 20x scrub speed
            let delta = self.scrub_direction as f64 * steps as f64 * 2.0;
            Some(delta)
        } else {
            None
        }
    }

    /// Returns true when the current track has finished playing naturally.
    pub fn is_finished(&self) -> bool {
        if self.seeking || self.scrub_direction != 0 || self.state != PlayState::Playing {
            return false;
        }
        // Check if the sink has drained (track played through)
        if self.sink.empty() {
            return true;
        }
        // Also check if we've exceeded total duration as a safety net
        if let Some(total) = self.total_duration {
            if self.get_elapsed() >= total {
                return true;
            }
        }
        false
    }

    pub fn playing(&self) -> bool {
        self.state == PlayState::Playing
    }
}

/// A rodio::Source backed by symphonia decoding with optional seek support.
/// Decodes the entire file into memory for simple, reliable playback.
pub struct SymphoniaSource {
    channels: u16,
    sample_rate: u32,
    duration: Option<Duration>,
    samples: Vec<f32>,
    position: usize,
}

impl SymphoniaSource {
    pub fn new(path: &Path) -> Result<Self> {
        Self::decode(path, None)
    }

    pub fn new_with_seek(path: &Path, seek_seconds: f64) -> Result<Self> {
        Self::decode(path, Some(seek_seconds))
    }

    fn decode(path: &Path, seek_to: Option<f64>) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Cannot open file: {}", path.display()))?;

        let mss = MediaSourceStream::new(
            Box::new(file),
            Default::default(),
        );

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let probed = symphonia::default::get_probe().format(
            &hint,
            mss,
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|e| crate::error::AppError::Decode(e.to_string()))?;

        let mut format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| crate::error::AppError::Decode("no default track".into()))?;

        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count() as u16).unwrap_or(2);
        let codec_params = track.codec_params.clone();

        // Duration from metadata
        let duration = track
            .codec_params
            .time_base
            .and_then(|tb| {
                track.codec_params.n_frames.map(|frames| {
                    let secs = frames as f64 * tb.numer as f64 / tb.denom as f64;
                    Duration::from_secs_f64(secs)
                })
            });

        let mut decoder = symphonia::default::get_codecs().make(
            &codec_params,
            &DecoderOptions::default(),
        )
        .map_err(|e| crate::error::AppError::Decode(e.to_string()))?;

        // Decode the entire file into memory
        let mut all_samples = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(_) => break,
            };

            let decoded = match decoder.decode(&packet) {
                Ok(d) => d,
                Err(_) => continue,
            };

            let spec = *decoded.spec();
            let num_frames = decoded.frames();
            let mut sample_buf = SampleBuffer::<f32>::new(num_frames as u64, spec);
            sample_buf.copy_interleaved_ref(decoded);

            all_samples.extend_from_slice(sample_buf.samples());
        }

        // Calculate seek offset in samples (always decode full file, then skip)
        let position = if let Some(seek_secs) = seek_to {
            if seek_secs > 0.0 {
                let offset = (seek_secs * sample_rate as f64 * channels as f64) as usize;
                offset.min(all_samples.len())
            } else {
                0
            }
        } else {
            0
        };

        Ok(Self {
            channels,
            sample_rate,
            duration,
            samples: all_samples,
            position,
        })
    }
}

impl Iterator for SymphoniaSource {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.position < self.samples.len() {
            let sample = self.samples[self.position];
            self.position += 1;
            Some(sample)
        } else {
            None
        }
    }
}

impl RodioSource for SymphoniaSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some((self.samples.len() - self.position) / self.channels as usize)
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }
}
