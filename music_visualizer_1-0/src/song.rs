use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use rubato::{FftFixedInOut, Resampler};
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct Song {
    is_playing: bool,
    audio_stream: Option<cpal::Stream>,
    audio_data: Arc<Mutex<Vec<f32>>>,
    current_frame: usize,
    pub title: String,
    pub filename: String,
}

impl Song {
    pub fn from_file(song_file_name: &str) -> Self {
        let song_path = format!("music_library/{}", song_file_name);
        let (raw_samples, file_sample_rate) = match Self::load_wav(&song_path) {
            Ok((data, rate)) => (data, rate),
            Err(e) => {
                eprintln!("Failed to load audio file '{}': {}", song_file_name, e);
                (Vec::new(), 44100)
            }
        };

        let device_sample_rate = Self::get_device_sample_rate().unwrap_or(48000);
        let channels = 2; // Assuming stereo

        let audio_data = if file_sample_rate != device_sample_rate {
            Arc::new(Mutex::new(Self::resample_to_device_rate(
                raw_samples,
                file_sample_rate,
                device_sample_rate,
                channels,
            )))
        } else {
            Arc::new(Mutex::new(raw_samples))
        };

        Song {
            is_playing: false,
            audio_stream: None,
            audio_data,
            current_frame: 0,
            title: Self::get_title_from_file(song_file_name),
            filename: song_file_name.to_string(),
        }
    }

    pub fn empty() -> Self {
        Song {
            is_playing: false,
            audio_stream: None,
            audio_data: Arc::new(Mutex::new(Vec::new())),
            current_frame: 0,
            title: "".to_string(),
            filename: "".to_string(),
        }
    }

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

    pub fn update(&mut self, should_play: bool) {
        if should_play && !self.is_playing {
            self.play();
        } else if !should_play && self.is_playing {
            self.pause();
        }
        self.is_playing = should_play;
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

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

        let sample_format = supported_config.sample_format();
        let mut config = supported_config.config();

        let audio_data = self.audio_data.clone();
        let frame_count = Arc::new(Mutex::new(self.current_frame));

        let stream_result = match sample_format {
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
                eprintln!("❌ Unsupported sample format: {:?}", sample_format);
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

    fn pause(&mut self) {
        if let Some(stream) = self.audio_stream.take() {
            drop(stream);
        }
    }

    fn load_wav(path: &str) -> Result<(Vec<f32>, u32), hound::Error> {
        let reader = hound::WavReader::open(Path::new(path))?;
        let spec = reader.spec();
        let samples: Vec<f32> = reader
            .into_samples::<i16>()
            .map(|s| s.unwrap_or(0) as f32 / i16::MAX as f32)
            .collect();
        Ok((samples, spec.sample_rate))
    }

    pub fn is_empty(&self) -> bool {
        self.audio_data.lock().unwrap().is_empty()
    }

    fn get_device_sample_rate() -> Option<u32> {
        let host = cpal::default_host();
        let device = host.default_output_device()?;
        let config = device.default_output_config().ok()?;
        Some(config.sample_rate().0)
    }

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

    fn resample_to_device_rate(
        input: Vec<f32>,
        from_rate: u32,
        to_rate: u32,
        channels: usize,
    ) -> Vec<f32> {
        println!("⚠️  Resampling from {} Hz to {} Hz...", from_rate, to_rate);
        let chunk_size = 1024;

        let mut resampler =
            FftFixedInOut::<f32>::new(from_rate as usize, to_rate as usize, chunk_size, channels)
                .expect("Failed to create Rubato resampler");

        let mut input_per_channel = vec![Vec::new(); channels];
        for (i, sample) in input.iter().enumerate() {
            input_per_channel[i % channels].push(*sample);
        }

        let input_frames = resampler.input_frames_next();
        let mut all_resampled = vec![Vec::new(); channels];

        for chunk_start in (0..input_per_channel[0].len()).step_by(input_frames) {
            let mut chunk: Vec<Vec<f32>> = input_per_channel
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

        let mut interleaved = Vec::with_capacity(all_resampled[0].len() * channels);
        for i in 0..all_resampled[0].len() {
            for ch in 0..channels {
                interleaved.push(all_resampled[ch][i]);
            }
        }

        println!("✅ Resampled to {} samples.", interleaved.len());
        interleaved
    }
}
