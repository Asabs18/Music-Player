//! Song module
//!
//! Handles loading, playing, and (if necessary) resampling of song audio data.
//! It supports dynamic sample rate selection based on the output device's capabilities,
//! caching a resampled file so that the expensive processing is only done once.

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{FftFixedInOut, Resampler};
use std::convert::TryInto;
use std::fs;
use std::io;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Represents a song that can be played.
///
/// This struct holds the audio data, playback state, and related metadata (such as title and file name).
/// It also maintains a reference to an output stream to play the song.
pub struct Song {
    is_playing: bool,
    audio_stream: Option<cpal::Stream>,
    audio_data: Arc<Mutex<Vec<f32>>>,
    current_frame: usize,
    /// The title of the song.
    pub title: String,
    /// The file name of the song.
    pub filename: String,
    /// The sample rate at which the audio data will be played.
    final_sample_rate: u32,
}

impl Song {
    // ============================================================================
    // Public Methods
    // ============================================================================

    /// Creates a `Song` from a file.
    ///
    /// This method loads a WAV file from the music library and always attempts to load the resampled
    /// version from cache in the `"music_cache"` folder. If the cached version does not exist, it loads
    /// the original file, resamples (if necessary), saves it to cache, then returns the processed audio.
    ///
    /// # Arguments
    ///
    /// * `song_file_name` - The name of the song file (assumed to be located in the "music_library" directory).
    ///
    /// # Returns
    ///
    /// A new `Song` instance with the appropriate audio data, title, and final sample rate.
    pub fn from_file(song_file_name: &str) -> Self {
        let song_path = format!("music_library/{}", song_file_name);

        // Load the file's native audio data and sample rate.
        let (raw_samples, file_sample_rate) = match Self::load_wav(&song_path) {
            Ok((data, rate)) => (data, rate),
            Err(e) => {
                eprintln!("Failed to load audio file '{}': {}", song_file_name, e);
                (Vec::new(), 44100)
            }
        };

        // Process the audio data from cache if available or create the cached version if needed.
        let (audio_data, final_rate) =
            Self::prepare_audio_data(song_file_name, raw_samples, file_sample_rate);

        Song {
            is_playing: false,
            audio_stream: None,
            audio_data,
            current_frame: 0,
            title: Self::get_title_from_file(song_file_name),
            filename: song_file_name.to_string(),
            final_sample_rate: final_rate,
        }
    }

    /// Returns an empty `Song` instance.
    ///
    /// Useful as a default when no song is selected.
    pub fn empty() -> Self {
        Song {
            is_playing: false,
            audio_stream: None,
            audio_data: Arc::new(Mutex::new(Vec::new())),
            current_frame: 0,
            title: "".to_string(),
            filename: "".to_string(),
            final_sample_rate: 44100,
        }
    }

    /// Converts a file name into a song title.
    ///
    /// # Arguments
    ///
    /// * `song_file_name` - The file name of the song.
    ///
    /// # Returns
    ///
    /// A formatted title string.
    pub fn get_title_from_file(song_file_name: &str) -> String {
        let mut title = song_file_name.to_string();
        if let Some(index) = title.rfind('.') {
            title.truncate(index);
        }
        title
            .split('-')
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Converts a song title into its corresponding file name.
    ///
    /// # Arguments
    ///
    /// * `title` - The song title.
    ///
    /// # Returns
    ///
    /// The file name (e.g. "example-song.wav").
    pub fn get_file_from_title(title: &str) -> String {
        format!(
            "{}.wav",
            title
                .split_whitespace()
                .map(|word| word.to_lowercase())
                .collect::<Vec<String>>()
                .join("-")
        )
    }

    /// Updates the playing state of the song.
    ///
    /// If `should_play` is true and the song is not already playing, playback starts.
    /// If false, playback is paused.
    ///
    /// # Arguments
    ///
    /// * `should_play` - Boolean flag indicating desired playing state.
    pub fn update(&mut self, should_play: bool) {
        if should_play && !self.is_playing {
            self.play();
        } else if !should_play && self.is_playing {
            self.pause();
        }
        self.is_playing = should_play;
    }

    /// Returns whether the song is currently playing.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Returns whether the song has no audio data.
    pub fn is_empty(&self) -> bool {
        self.audio_data.lock().unwrap().is_empty()
    }

    /// Outputs debug information regarding the output device's supported configurations.
    pub fn debug_info(&self) {
        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("No output device available.");
                return;
            }
        };
        self.debug_supported_configs(&device);
    }

    // ============================================================================
    // Private Helper Methods
    // ============================================================================

    /// Prepares the audio data by attempting to load the cached file first.
    ///
    /// If the cached resampled file (located in "music_cache") exists, it is loaded.
    /// Otherwise, this function processes the original audio data (resampling when necessary),
    /// saves it to cache, and returns the processed data.
    ///
    /// # Arguments
    ///
    /// * `song_file_name` - The original song file name.
    /// * `raw_samples` - The raw audio samples loaded from the original file.
    /// * `file_sample_rate` - The native sample rate of the file.
    ///
    /// # Returns
    ///
    /// A tuple containing:
    /// - `Arc<Mutex<Vec<f32>>>` wrapping the processed (cached) audio samples.
    /// - The final sample rate to use.
    fn prepare_audio_data(
        song_file_name: &str,
        raw_samples: Vec<f32>,
        file_sample_rate: u32,
    ) -> (Arc<Mutex<Vec<f32>>>, u32) {
        let (supports_native_rate, final_rate) =
            Self::determine_final_sample_rate(file_sample_rate);
        // Construct the cache path, naming it with the song title and the final sample rate.
        let cache_path = format!(
            "music_cache/{}-{}Hz.wav",
            Self::get_title_from_file(song_file_name),
            final_rate
        );
        // If a cached file exists, always prefer loading it.
        if Path::new(&cache_path).exists() {
            match Self::load_wav(&cache_path) {
                Ok((cached_samples, _)) => {
                    return (Arc::new(Mutex::new(cached_samples)), final_rate);
                }
                Err(e) => {
                    eprintln!(
                        "Failed to load cached file '{}': {}. Will process original file...",
                        cache_path, e
                    );
                }
            }
        }
        // No cache exists; process the original file.
        // Even if the device supports the native rate, we choose to use the cache version.
        let channels = 2; // assuming stereo
        let processed = if file_sample_rate != final_rate {
            Self::resample_and_cache(
                raw_samples,
                file_sample_rate,
                final_rate,
                channels,
                &cache_path,
            )
        } else {
            // If no resampling is needed, copy the file into cache.
            if let Err(e) = Self::save_wav(
                &cache_path,
                &raw_samples,
                final_rate,
                channels.try_into().unwrap(),
            ) {
                eprintln!("Warning: Could not save cache to '{}': {}", cache_path, e);
            }
            raw_samples
        };
        (Arc::new(Mutex::new(processed)), final_rate)
    }

    /// Determines the final sample rate for playing a song.
    ///
    /// If the output device supports the native sample rate, it is used.
    /// Otherwise, the device's default sample rate is used as a fallback.
    ///
    /// # Arguments
    ///
    /// * `file_sample_rate` - Native sample rate of the file.
    ///
    /// # Returns
    ///
    /// A tuple `(bool, u32)` where the boolean indicates if the native rate is supported,
    /// and the `u32` is the final sample rate to use.
    fn determine_final_sample_rate(file_sample_rate: u32) -> (bool, u32) {
        let host = cpal::default_host();
        let device = host.default_output_device();
        let mut supports_native_rate = false;
        let mut fallback_rate = file_sample_rate;

        if let Some(device) = &device {
            match device.supported_output_configs() {
                Ok(configs) => {
                    for config in configs {
                        if file_sample_rate >= config.min_sample_rate().0
                            && file_sample_rate <= config.max_sample_rate().0
                        {
                            supports_native_rate = true;
                            break;
                        }
                    }
                    if !supports_native_rate {
                        if let Ok(default_config) = device.default_output_config() {
                            fallback_rate = default_config.sample_rate().0;
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Error querying supported configs: {}. Assuming the song rate is unsupported.",
                        e
                    );
                }
            }
        }
        (supports_native_rate, fallback_rate)
    }

    /// Resamples the input audio data to the target sample rate and saves the result to cache.
    ///
    /// # Arguments
    ///
    /// * `input` - The original audio samples.
    /// * `from_rate` - The native sample rate.
    /// * `to_rate` - The target sample rate.
    /// * `channels` - Number of audio channels.
    /// * `cache_path` - The file path where the resampled data should be saved.
    ///
    /// # Returns
    ///
    /// A `Vec<f32>` containing the resampled and interleaved audio data.
    fn resample_and_cache(
        input: Vec<f32>,
        from_rate: u32,
        to_rate: u32,
        channels: usize,
        cache_path: &str,
    ) -> Vec<f32> {
        println!("⚠️ Resampling from {} Hz to {} Hz...", from_rate, to_rate);
        let resampled = Self::resample_to_device_rate(input, from_rate, to_rate, channels);
        println!("✅ Resampled to {} samples.", resampled.len());
        if let Err(e) = Self::save_wav(
            cache_path,
            &resampled,
            to_rate,
            channels.try_into().unwrap(),
        ) {
            eprintln!("Warning: Could not save cache to '{}': {}", cache_path, e);
        }
        resampled
    }

    /// Starts playback by creating and starting an output stream.
    ///
    /// The stream is configured to use `final_sample_rate`. If a stream is already active, it does nothing.
    fn play(&mut self) {
        if self.audio_stream.is_some() {
            return;
        }

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(d) => d,
            None => {
                eprintln!("❌ No output device available.");
                return;
            }
        };

        let supported_config = match device.default_output_config() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("❌ Failed to get default output config: {}", e);
                return;
            }
        };

        let mut config = supported_config.config();
        config.sample_rate = cpal::SampleRate(self.final_sample_rate);

        let audio_data = self.audio_data.clone();
        let frame_count = Arc::new(Mutex::new(self.current_frame));

        let stream_result = match supported_config.sample_format() {
            cpal::SampleFormat::F32 => device.build_output_stream(
                &config,
                {
                    let frame_count = Arc::clone(&frame_count);
                    move |data: &mut [f32], _| {
                        let audio_data = audio_data.lock().unwrap();
                        let mut count = frame_count.lock().unwrap();
                        for sample in data.iter_mut() {
                            *sample = if *count < audio_data.len() {
                                audio_data[*count]
                            } else {
                                0.0
                            };
                            *count += 1;
                        }
                    }
                },
                move |err| eprintln!("⚠️ Stream error: {}", err),
                None,
            ),
            _ => {
                eprintln!(
                    "❌ Unsupported sample format: {:?}",
                    supported_config.sample_format()
                );
                return;
            }
        };

        match stream_result {
            Ok(stream) => {
                if let Err(e) = stream.play() {
                    eprintln!("❌ Failed to start playback: {}", e);
                } else {
                    println!("✅ Playback started.");
                }
                self.audio_stream = Some(stream);
            }
            Err(e) => {
                eprintln!("❌ Stream creation failed: {}", e);
                self.debug_supported_configs(&device);
            }
        }
    }

    /// Pauses playback by dropping the current output stream.
    fn pause(&mut self) {
        if let Some(stream) = self.audio_stream.take() {
            drop(stream);
        }
    }

    /// Loads WAV audio data using the hound crate.
    ///
    /// # Arguments
    ///
    /// * `path` - The file system path to the WAV file.
    ///
    /// # Returns
    ///
    /// A `Result` with a tuple of the audio samples and the sample rate on success, or a `hound::Error`.
    fn load_wav(path: &str) -> Result<(Vec<f32>, u32), hound::Error> {
        let reader = hound::WavReader::open(Path::new(path))?;
        let spec = reader.spec();
        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .map(|s| s.unwrap_or(0) as f32 / i16::MAX as f32)
            .collect();
        Ok((samples, spec.sample_rate))
    }

    /// Outputs the supported configurations for the given output device.
    ///
    /// # Arguments
    ///
    /// * `device` - A reference to the output device.
    fn debug_supported_configs(&self, device: &cpal::Device) {
        println!(
            "🧪 Supported configs for device '{}':",
            device.name().unwrap_or_default()
        );
        if let Ok(configs) = device.supported_output_configs() {
            for cfg in configs {
                println!(
                    "  - {:?}, channels: {}, rate: {}-{}",
                    cfg.sample_format(),
                    cfg.channels(),
                    cfg.min_sample_rate().0,
                    cfg.max_sample_rate().0
                );
            }
        } else {
            eprintln!("⚠️ Could not retrieve supported output configs.");
        }
    }

    /// Resamples audio data from one sample rate to another using Rubato.
    ///
    /// This method de-interleaves the input samples, processes them in chunks, and then re-interleaves the
    /// resampled data.
    ///
    /// # Arguments
    ///
    /// * `input` - The original audio samples.
    /// * `from_rate` - The native sample rate.
    /// * `to_rate` - The target sample rate.
    /// * `channels` - The number of audio channels.
    ///
    /// # Returns
    ///
    /// A vector of interleaved, resampled audio samples.
    fn resample_to_device_rate(
        input: Vec<f32>,
        from_rate: u32,
        to_rate: u32,
        channels: usize,
    ) -> Vec<f32> {
        let chunk_size = 1024;
        let mut resampler =
            FftFixedInOut::<f32>::new(from_rate as usize, to_rate as usize, chunk_size, channels)
                .expect("Failed to create Rubato resampler");

        // De-interleave samples.
        let mut input_per_channel = vec![Vec::new(); channels];
        for (i, sample) in input.iter().enumerate() {
            input_per_channel[i % channels].push(*sample);
        }

        let input_frames = resampler.input_frames_next();
        let mut all_resampled = vec![Vec::new(); channels];

        // Process in chunks.
        for chunk_start in (0..input_per_channel[0].len()).step_by(input_frames) {
            let chunk: Vec<Vec<f32>> = input_per_channel
                .iter()
                .map(|channel| {
                    let mut slice = vec![0.0; input_frames];
                    for i in 0..input_frames {
                        if let Some(&s) = channel.get(chunk_start + i) {
                            slice[i] = s;
                        }
                    }
                    slice
                })
                .collect();

            let resampled = resampler.process(&chunk, None).expect("Resampling failed");
            for (i, chan) in resampled.into_iter().enumerate() {
                all_resampled[i].extend(chan);
            }
        }

        // Re-interleave samples.
        let mut interleaved = Vec::with_capacity(all_resampled[0].len() * channels);
        for i in 0..all_resampled[0].len() {
            for ch in 0..channels {
                interleaved.push(all_resampled[ch][i]);
            }
        }
        interleaved
    }

    /// Saves the provided audio samples as a WAV file using the hound crate.
    ///
    /// # Arguments
    ///
    /// * `path` - The file path where the WAV file should be saved.
    /// * `samples` - The audio samples to save.
    /// * `sample_rate` - The sample rate of the audio data.
    /// * `channels` - The number of audio channels.
    ///
    /// # Returns
    ///
    /// A `Result` which is `Ok(())` on success or a `hound::Error` on failure.
    fn save_wav(
        path: &str,
        samples: &[f32],
        sample_rate: u32,
        channels: u16,
    ) -> Result<(), hound::Error> {
        // Ensure the cache directory exists.
        if let Some(parent) = Path::new(path).parent() {
            fs::create_dir_all(parent).ok();
        }

        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(path, spec)?;
        for sample in samples {
            let scaled = (sample * i16::MAX as f32) as i16;
            writer.write_sample(scaled)?;
        }
        writer.finalize()?;
        Ok(())
    }
}
