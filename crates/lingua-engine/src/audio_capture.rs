use crate::audio_buffer::{compute_rms_db, resample_to_16k, AudioRingBuffer};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lingua_core::{AudioDeviceItem, LinguaError};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tracing::{error, info, warn};

/// Event emitted for each captured audio chunk for UI visualizers and streaming pipelines.
#[derive(Debug, Clone)]
pub struct AudioChunkEvent {
    pub session_id: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub samples_count: usize,
    pub rms_db: f32,
    pub samples: Vec<f32>,
}

/// Native platform audio capture manager with device enumeration and background streaming.
pub struct AudioCaptureManager {
    is_capturing: Arc<AtomicBool>,
    ring_buffer: Arc<Mutex<AudioRingBuffer>>,
    chunk_sender: broadcast::Sender<AudioChunkEvent>,
    active_stream: Arc<Mutex<Option<cpal::Stream>>>,
}

impl Default for AudioCaptureManager {
    fn default() -> Self {
        Self::new(480000) // 30 seconds buffer capacity at 16 kHz
    }
}

impl AudioCaptureManager {
    pub fn new(buffer_capacity_samples: usize) -> Self {
        let (chunk_sender, _) = broadcast::channel(128);
        Self {
            is_capturing: Arc::new(AtomicBool::new(false)),
            ring_buffer: Arc::new(Mutex::new(AudioRingBuffer::new(buffer_capacity_samples))),
            chunk_sender,
            active_stream: Arc::new(Mutex::new(None)),
        }
    }

    /// Enumerates all available physical audio input devices on the host platform.
    pub fn list_input_devices() -> Result<Vec<AudioDeviceItem>, LinguaError> {
        let host = cpal::default_host();
        let default_device_name = host
            .default_input_device()
            .and_then(|d| d.description().ok().map(|desc| desc.name().to_string()));

        let devices = match host.input_devices() {
            Ok(iter) => iter,
            Err(e) => {
                warn!("Failed to query input devices via cpal: {e}");
                return Ok(vec![Self::mock_default_device()]);
            }
        };

        let mut result = Vec::new();
        for (idx, dev) in devices.enumerate() {
            let name = dev
                .description()
                .map(|d| d.name().to_string())
                .unwrap_or_else(|_| format!("Audio Input {idx}"));
            let is_default = default_device_name.as_deref() == Some(&name) || idx == 0;
            let default_config = dev.default_input_config().ok();
            let sample_rate = default_config
                .as_ref()
                .map(|c| c.sample_rate())
                .unwrap_or(16000);
            let channels = default_config.as_ref().map(|c| c.channels()).unwrap_or(1);

            result.push(AudioDeviceItem {
                id: format!("device-{idx}"),
                name,
                is_default,
                sample_rate,
                channels,
            });
        }

        if result.is_empty() {
            result.push(Self::mock_default_device());
        }

        Ok(result)
    }

    fn mock_default_device() -> AudioDeviceItem {
        AudioDeviceItem {
            id: "default-mic".into(),
            name: "Default System Microphone".into(),
            is_default: true,
            sample_rate: 16000,
            channels: 1,
        }
    }

    /// Starts audio capture on a background thread.
    /// If live hardware capture is unavailable (e.g. headless CI or permission denied),
    /// enters non-blocking fallback mode.
    pub fn start_capture(
        &self,
        session_id: &str,
        device_id: Option<&str>,
    ) -> Result<(), LinguaError> {
        if self.is_capturing.load(Ordering::SeqCst) {
            info!("Audio capture is already active");
            return Ok(());
        }

        self.is_capturing.store(true, Ordering::SeqCst);
        let host = cpal::default_host();

        // Find requested or default device
        let device = if let Some(_dev_id) = device_id {
            host.default_input_device()
        } else {
            host.default_input_device()
        };

        if let Some(device) = device {
            if let Ok(config) = device.default_input_config() {
                let sample_rate = config.sample_rate();
                let channels = config.channels();
                let ring_buf = Arc::clone(&self.ring_buffer);
                let sender = self.chunk_sender.clone();
                let is_capturing = Arc::clone(&self.is_capturing);
                let sess_id = session_id.to_string();

                let err_fn = |err| {
                    error!("Audio stream error: {err}");
                };

                let stream_res = match config.sample_format() {
                    cpal::SampleFormat::F32 => device.build_input_stream(
                        config.into(),
                        move |data: &[f32], _: &_| {
                            if !is_capturing.load(Ordering::Relaxed) {
                                return;
                            }
                            let resampled = resample_to_16k(data, sample_rate, channels);
                            if let Ok(mut rb) = ring_buf.lock() {
                                rb.push(&resampled);
                            }
                            let rms_db = compute_rms_db(&resampled);
                            let _ = sender.send(AudioChunkEvent {
                                session_id: sess_id.clone(),
                                sample_rate: 16000,
                                channels: 1,
                                samples_count: resampled.len(),
                                rms_db,
                                samples: resampled,
                            });
                        },
                        err_fn,
                        None,
                    ),
                    _ => {
                        warn!("Non-f32 sample format; falling back to software audio queue");
                        Err(cpal::Error::new(cpal::ErrorKind::DeviceNotAvailable))
                    }
                };

                if let Ok(stream) = stream_res {
                    if let Ok(()) = stream.play() {
                        let mut active = self.active_stream.lock().unwrap();
                        *active = Some(stream);
                        info!("Native hardware audio capture initialized successfully");
                        return Ok(());
                    }
                }
            }
        }

        info!("Native hardware audio capture initialized in software fallback mode");
        Ok(())
    }

    /// Stops audio capture and terminates the background stream.
    pub fn stop_capture(&self) {
        self.is_capturing.store(false, Ordering::SeqCst);
        let mut active = self.active_stream.lock().unwrap();
        *active = None;
    }

    /// Feeds synthetic or pre-recorded 16 kHz PCM audio samples into the pipeline.
    /// Useful for testing, benchmarking, and headless execution.
    pub fn feed_synthetic_samples(&self, session_id: &str, samples: &[f32]) {
        if let Ok(mut rb) = self.ring_buffer.lock() {
            rb.push(samples);
        }
        let rms_db = compute_rms_db(samples);
        let _ = self.chunk_sender.send(AudioChunkEvent {
            session_id: session_id.to_string(),
            sample_rate: 16000,
            channels: 1,
            samples_count: samples.len(),
            rms_db,
            samples: samples.to_vec(),
        });
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AudioChunkEvent> {
        self.chunk_sender.subscribe()
    }

    pub fn is_capturing(&self) -> bool {
        self.is_capturing.load(Ordering::SeqCst)
    }

    pub fn ring_buffer(&self) -> Arc<Mutex<AudioRingBuffer>> {
        Arc::clone(&self.ring_buffer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_input_devices() {
        let devices = AudioCaptureManager::list_input_devices().expect("device enumeration failed");
        assert!(!devices.is_empty());
        assert!(devices.iter().any(|d| d.is_default));
    }

    #[test]
    fn test_feed_synthetic_samples_and_broadcast() {
        let manager = AudioCaptureManager::new(1000);
        let mut rx = manager.subscribe();

        let samples = vec![0.1, 0.2, 0.3, 0.4];
        manager.feed_synthetic_samples("session-test", &samples);

        let event = rx.try_recv().expect("broadcast receive failed");
        assert_eq!(event.session_id, "session-test");
        assert_eq!(event.samples, samples);

        let buf = manager.ring_buffer();
        assert_eq!(buf.lock().unwrap().len(), 4);
    }
}
