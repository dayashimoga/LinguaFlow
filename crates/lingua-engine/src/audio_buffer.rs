use std::collections::VecDeque;

/// Thread-safe bounded ring buffer for 16 kHz 32-bit floating point PCM audio samples.
/// Enforces hard capacity limits to prevent memory accumulation during extended speech sessions.
#[derive(Debug, Clone)]
pub struct AudioRingBuffer {
    buffer: VecDeque<f32>,
    max_capacity_samples: usize,
    total_samples_written: u64,
    dropped_samples: u64,
}

impl AudioRingBuffer {
    /// Creates a new `AudioRingBuffer` with a fixed sample capacity.
    /// For 16 kHz mono audio:
    /// - 16,000 samples = 1.0 second
    /// - 480,000 samples = 30.0 seconds
    pub fn new(max_capacity_samples: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(max_capacity_samples.min(65536)),
            max_capacity_samples,
            total_samples_written: 0,
            dropped_samples: 0,
        }
    }

    /// Pushes incoming PCM samples into the buffer.
    /// If the buffer exceeds capacity, the oldest samples are evicted to preserve bounds.
    pub fn push(&mut self, samples: &[f32]) {
        for &sample in samples {
            if self.buffer.len() >= self.max_capacity_samples {
                self.buffer.pop_front();
                self.dropped_samples += 1;
            }
            // Clamp sample to valid normalized PCM range [-1.0, 1.0]
            let clamped = sample.clamp(-1.0, 1.0);
            self.buffer.push_back(clamped);
            self.total_samples_written += 1;
        }
    }

    /// Drains all available samples from the buffer.
    pub fn drain_all(&mut self) -> Vec<f32> {
        self.buffer.drain(..).collect()
    }

    /// Reads up to `count` samples from the front without consuming them.
    pub fn peek_window(&self, count: usize) -> Vec<f32> {
        self.buffer.iter().take(count).copied().collect()
    }

    /// Consumes up to `count` samples from the front of the buffer.
    pub fn read_window(&mut self, count: usize) -> Vec<f32> {
        let drain_count = count.min(self.buffer.len());
        self.buffer.drain(..drain_count).collect()
    }

    /// Returns the number of samples currently in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Returns true if the buffer has no samples.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Clears all samples in the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn dropped_samples_count(&self) -> u64 {
        self.dropped_samples
    }

    pub fn total_samples_written(&self) -> u64 {
        self.total_samples_written
    }
}

/// Downmixes and resamples an arbitrary incoming audio stream to 16,000 Hz mono PCM.
pub fn resample_to_16k(input: &[f32], input_rate: u32, channels: u16) -> Vec<f32> {
    if input.is_empty() {
        return Vec::new();
    }

    // Step 1: Downmix multi-channel to mono
    let mono: Vec<f32> = if channels > 1 {
        let ch = channels as usize;
        input
            .chunks_exact(ch)
            .map(|frame| frame.iter().sum::<f32>() / (ch as f32))
            .collect()
    } else {
        input.to_vec()
    };

    if input_rate == 16000 {
        return mono;
    }

    // Step 2: Linear interpolation resampling to 16,000 Hz
    let target_rate = 16000.0_f64;
    let ratio = input_rate as f64 / target_rate;
    let target_len = (mono.len() as f64 / ratio).ceil() as usize;
    let mut resampled = Vec::with_capacity(target_len);

    for i in 0..target_len {
        let src_idx = i as f64 * ratio;
        let idx0 = src_idx.floor() as usize;
        let idx1 = (idx0 + 1).min(mono.len().saturating_sub(1));
        let frac = (src_idx - idx0 as f64) as f32;

        let val0 = mono.get(idx0).copied().unwrap_or(0.0);
        let val1 = mono.get(idx1).copied().unwrap_or(0.0);
        resampled.push(val0 + frac * (val1 - val0));
    }

    resampled
}

/// Computes the Root-Mean-Square (RMS) volume level of audio samples in Decibels Full Scale (dBFS).
/// Returns a value between -96.0 dB (silence) and 0.0 dB (maximum loudness).
pub fn compute_rms_db(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return -96.0;
    }

    let sum_sq: f32 = samples.iter().map(|&s| s * s).sum();
    let mean_sq = sum_sq / (samples.len() as f32);
    let rms = mean_sq.sqrt();

    if rms <= 1e-5 {
        -96.0
    } else {
        (20.0 * rms.log10()).clamp(-96.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_ring_buffer_bounds_and_overflow() {
        let mut rb = AudioRingBuffer::new(100);
        assert_eq!(rb.len(), 0);
        assert!(rb.is_empty());

        let samples: Vec<f32> = (0..80).map(|i| i as f32 / 100.0).collect();
        rb.push(&samples);
        assert_eq!(rb.len(), 80);
        assert_eq!(rb.dropped_samples_count(), 0);

        // Push 50 more samples, which should cause 30 drops to maintain cap of 100
        let more_samples: Vec<f32> = (0..50).map(|i| (i + 100) as f32 / 200.0).collect();
        rb.push(&more_samples);
        assert_eq!(rb.len(), 100);
        assert_eq!(rb.dropped_samples_count(), 30);

        let drained = rb.drain_all();
        assert_eq!(drained.len(), 100);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_resampling_48k_to_16k() {
        let input_48k = vec![0.0, 0.5, 1.0, 0.5, 0.0, -0.5, -1.0, -0.5];
        let resampled = resample_to_16k(&input_48k, 48000, 1);
        assert!(!resampled.is_empty());
        assert!(resampled.len() < input_48k.len());
    }

    #[test]
    fn test_rms_db_silence_and_signal() {
        let silence = vec![0.0; 512];
        assert_eq!(compute_rms_db(&silence), -96.0);

        let loud = vec![1.0; 512];
        assert!(compute_rms_db(&loud) >= -0.1);
    }
}
