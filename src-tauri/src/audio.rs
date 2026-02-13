use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use thiserror::Error;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const TARGET_SAMPLE_RATE: u32 = 16_000;
const MIN_SAMPLES: usize = 3_200; // 0.2s at 16kHz

#[derive(Debug, Error)]
pub(crate) enum AudioError {
    #[error("no input device available")]
    NoInputDevice,
    #[error("no supported input config: {0}")]
    NoSupportedConfig(String),
    #[error("failed to build input stream: {0}")]
    BuildStream(String),
    #[error("failed to start stream: {0}")]
    PlayStream(String),
}

impl serde::Serialize for AudioError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub(crate) struct AudioRecorder {
    stop_signal: Arc<AtomicBool>,
    samples: Arc<Mutex<Vec<f32>>>,
    thread_handle: Option<std::thread::JoinHandle<Result<(), AudioError>>>,
}

impl AudioRecorder {
    pub(crate) fn new() -> Self {
        Self {
            stop_signal: Arc::new(AtomicBool::new(false)),
            samples: Arc::new(Mutex::new(Vec::new())),
            thread_handle: None,
        }
    }

    pub(crate) fn start(&mut self) -> Result<(), AudioError> {
        self.stop_signal.store(false, Ordering::SeqCst);
        self.samples.lock().expect("samples mutex poisoned").clear();

        let stop_signal = Arc::clone(&self.stop_signal);
        let samples = Arc::clone(&self.samples);

        let handle = std::thread::spawn(move || -> Result<(), AudioError> {
            let host = cpal::default_host();
            let device = host
                .default_input_device()
                .ok_or(AudioError::NoInputDevice)?;

            let supported_config = device
                .supported_input_configs()
                .map_err(|e| AudioError::NoSupportedConfig(e.to_string()))?
                .find(|c| c.channels() == 1 && c.sample_format() == cpal::SampleFormat::F32)
                .or_else(|| {
                    device
                        .supported_input_configs()
                        .ok()?
                        .find(|c| c.sample_format() == cpal::SampleFormat::F32)
                })
                .or_else(|| device.supported_input_configs().ok()?.next())
                .ok_or_else(|| {
                    AudioError::NoSupportedConfig("no compatible config found".to_string())
                })?;

            let min_rate = supported_config.min_sample_rate().0;
            let max_rate = supported_config.max_sample_rate().0;

            let device_rate = if min_rate <= TARGET_SAMPLE_RATE && TARGET_SAMPLE_RATE <= max_rate {
                TARGET_SAMPLE_RATE
            } else {
                supported_config.max_sample_rate().0
            };

            let channels = supported_config.channels();
            let config = cpal::StreamConfig {
                channels,
                sample_rate: cpal::SampleRate(device_rate),
                buffer_size: cpal::BufferSize::Default,
            };

            let needs_resample = device_rate != TARGET_SAMPLE_RATE;
            let rate_ratio = if needs_resample {
                TARGET_SAMPLE_RATE as f64 / device_rate as f64
            } else {
                1.0
            };

            let samples_clone = Arc::clone(&samples);
            let sample_format = supported_config.sample_format();

            let stream = match sample_format {
                cpal::SampleFormat::F32 => {
                    device.build_input_stream(
                        &config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            let mono: Vec<f32> = if channels > 1 {
                                data.chunks(channels as usize)
                                    .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                                    .collect()
                            } else {
                                data.to_vec()
                            };

                            let processed = if needs_resample {
                                resample_linear(&mono, rate_ratio)
                            } else {
                                mono
                            };

                            if let Ok(mut s) = samples_clone.lock() {
                                s.extend_from_slice(&processed);
                            }
                        },
                        |err| {
                            log::error!("audio stream error: {}", err);
                        },
                        None,
                    )
                }
                cpal::SampleFormat::I16 => {
                    device.build_input_stream(
                        &config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let float_data: Vec<f32> =
                                data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();

                            let mono: Vec<f32> = if channels > 1 {
                                float_data
                                    .chunks(channels as usize)
                                    .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                                    .collect()
                            } else {
                                float_data
                            };

                            let processed = if needs_resample {
                                resample_linear(&mono, rate_ratio)
                            } else {
                                mono
                            };

                            if let Ok(mut s) = samples_clone.lock() {
                                s.extend_from_slice(&processed);
                            }
                        },
                        |err| {
                            log::error!("audio stream error: {}", err);
                        },
                        None,
                    )
                }
                _ => {
                    return Err(AudioError::NoSupportedConfig(format!(
                        "unsupported sample format: {:?}",
                        sample_format
                    )));
                }
            };

            let stream = stream.map_err(|e| AudioError::BuildStream(e.to_string()))?;
            stream
                .play()
                .map_err(|e| AudioError::PlayStream(e.to_string()))?;

            // Keep recording until stop signal
            while !stop_signal.load(Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            drop(stream);
            Ok(())
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Returns a snapshot of the current audio samples without stopping recording.
    pub(crate) fn peek_samples(&self) -> Vec<f32> {
        self.samples
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    pub(crate) fn stop(&mut self) -> Vec<f32> {
        self.stop_signal.store(true, Ordering::SeqCst);

        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        let samples = self
            .samples
            .lock()
            .expect("samples mutex poisoned")
            .clone();

        // Short recording protection
        if samples.len() < MIN_SAMPLES {
            return Vec::new();
        }

        samples
    }
}

fn resample_linear(input: &[f32], ratio: f64) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }

    let output_len = (input.len() as f64 * ratio).ceil() as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_pos = i as f64 / ratio;
        let src_idx = src_pos as usize;
        let frac = (src_pos - src_idx as f64) as f32;

        let sample = if src_idx + 1 < input.len() {
            input[src_idx] * (1.0 - frac) + input[src_idx + 1] * frac
        } else if src_idx < input.len() {
            input[src_idx]
        } else {
            0.0
        };

        output.push(sample);
    }

    output
}
