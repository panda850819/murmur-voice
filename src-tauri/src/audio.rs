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
    #[error("failed to lock samples mutex: {0}")]
    LockPoisoned(String),
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
                    // Reusable buffers to avoid allocation in hot path
                    let mut intermediate = Vec::with_capacity(4096);
                    let mut output_buffer = Vec::with_capacity(4096);

                    device.build_input_stream(
                        &config,
                        move |data: &[f32], _: &cpal::InputCallbackInfo| {
                            // Optimization: Direct copy if 1 channel and no resample
                            if channels == 1 && !needs_resample {
                                if let Ok(mut s) = samples_clone.lock() {
                                    s.extend_from_slice(data);
                                }
                                return;
                            }

                            intermediate.clear();
                            output_buffer.clear();

                            // 1. Prepare source (convert to Mono F32 if needed)
                            let source = if channels > 1 {
                                // Downmix to mono
                                for frame in data.chunks(channels as usize) {
                                    let sum: f32 = frame.iter().sum();
                                    intermediate.push(sum / channels as f32);
                                }
                                &intermediate
                            } else {
                                // Already mono F32
                                data
                            };

                            // 2. Resample or pass through
                            if needs_resample {
                                resample_linear_into(source, rate_ratio, &mut output_buffer);
                                if let Ok(mut s) = samples_clone.lock() {
                                    s.extend_from_slice(&output_buffer);
                                }
                            } else {
                                // If we are here, channels > 1 but !needs_resample (so source is intermediate)
                                // or channels == 1 (handled by early return above).
                                if let Ok(mut s) = samples_clone.lock() {
                                    s.extend_from_slice(source);
                                }
                            }
                        },
                        |err| {
                            log::error!("audio stream error: {}", err);
                        },
                        None,
                    )
                }
                cpal::SampleFormat::I16 => {
                    // Reusable buffers
                    let mut intermediate = Vec::with_capacity(4096);
                    let mut output_buffer = Vec::with_capacity(4096);

                    device.build_input_stream(
                        &config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            intermediate.clear();
                            output_buffer.clear();

                            // 1. Convert to Mono F32
                            if channels > 1 {
                                for frame in data.chunks(channels as usize) {
                                    let sum: f32 = frame.iter().map(|&s| s as f32 / i16::MAX as f32).sum();
                                    intermediate.push(sum / channels as f32);
                                }
                            } else {
                                intermediate.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                            }

                            // 2. Resample or pass through
                            if needs_resample {
                                resample_linear_into(&intermediate, rate_ratio, &mut output_buffer);
                                if let Ok(mut s) = samples_clone.lock() {
                                    s.extend_from_slice(&output_buffer);
                                }
                            } else if let Ok(mut s) = samples_clone.lock() {
                                s.extend_from_slice(&intermediate);
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
    pub(crate) fn peek_samples(&self) -> Result<Vec<f32>, AudioError> {
        self.samples
            .lock()
            .map(|s| s.clone())
            .map_err(|e| AudioError::LockPoisoned(e.to_string()))
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

fn resample_linear_into(input: &[f32], ratio: f64, output: &mut Vec<f32>) {
    if input.is_empty() {
        return;
    }

    let output_len = (input.len() as f64 * ratio).ceil() as usize;
    output.reserve(output_len);

    // Optimization: Pre-calculate inverse ratio to use multiplication instead of division
    let inv_ratio = 1.0 / ratio;

    for i in 0..output_len {
        let src_pos = i as f64 * inv_ratio;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resample_linear_into_identity() {
        let input = vec![0.0, 0.5, 1.0];
        let mut output = Vec::new();
        resample_linear_into(&input, 1.0, &mut output);

        assert_eq!(output.len(), 3);
        assert!((output[0] - 0.0).abs() < 1e-6);
        assert!((output[1] - 0.5).abs() < 1e-6);
        assert!((output[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_resample_linear_into_upsample() {
        let input = vec![0.0, 1.0];
        // Ratio 2.0 -> output len 4
        let mut output = Vec::new();
        resample_linear_into(&input, 2.0, &mut output);

        assert_eq!(output.len(), 4);
        assert!((output[0] - 0.0).abs() < 1e-6); // idx 0
        assert!((output[1] - 0.5).abs() < 1e-6); // idx 0.5
        assert!((output[2] - 1.0).abs() < 1e-6); // idx 1.0
        // idx 1.5 -> src_idx 1. (idx+1 out of bounds). input[1] = 1.0.
        // 1.0 * (1-0.5) + (out_of_bounds? no, logic says if src_idx < len returns input[src_idx])
        // wait, logic says:
        // if src_idx + 1 < input.len() { lerp }
        // else if src_idx < input.len() { input[src_idx] }
        // else { 0.0 }

        // i=3. src_pos = 1.5. src_idx=1. frac=0.5.
        // src_idx+1 = 2. input len is 2. 2 < 2 is false.
        // src_idx < input.len() -> 1 < 2 is true.
        // returns input[1] which is 1.0.
        assert!((output[3] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_resample_linear_into_downsample() {
        let input = vec![0.0, 0.5, 1.0, 0.5];
        // Ratio 0.5 -> output len 2
        let mut output = Vec::new();
        resample_linear_into(&input, 0.5, &mut output);

        assert_eq!(output.len(), 2);
        // i=0. pos=0. input[0]=0.0
        assert!((output[0] - 0.0).abs() < 1e-6);

        // i=1. pos=2. input[2]=1.0
        assert!((output[1] - 1.0).abs() < 1e-6);
    }
}
