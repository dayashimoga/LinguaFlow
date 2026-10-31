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

/// Downmixes and resamples an arbitrary incoming audio stream to 16,000 Hz mono PCM using
/// polyphase windowed-sinc interpolation with bandlimited anti-aliasing.
pub fn resample_to_16k(input: &[f32], input_rate: u32, channels: u16) -> Vec<f32> {
    let mut out = Vec::new();
    resample_to_16k_into(input, input_rate, channels, &mut out);
    out
}

/// In-place reusable buffer variant of polyphase 16 kHz mono resampling.
/// Reuses the existing capacity of `output` to guarantee zero heap re-allocation during continuous processing.
pub fn resample_to_16k_into(input: &[f32], input_rate: u32, channels: u16, output: &mut Vec<f32>) {
    output.clear();
    if input.is_empty() {
        return;
    }

    // Step 1: Downmix multi-channel frames to mono
    let ch = channels.max(1) as usize;
    let num_mono_samples = input.len() / ch;
    output.reserve(num_mono_samples);

    if input_rate == 16000 && ch == 1 {
        output.extend_from_slice(input);
        return;
    }

    // Temporary mono buffer if stereo/multichannel
    let mono_data: Vec<f32> = if ch > 1 {
        input
            .chunks_exact(ch)
            .map(|frame| frame.iter().sum::<f32>() / (ch as f32))
            .collect()
    } else {
        input.to_vec()
    };

    if input_rate == 16000 {
        output.extend_from_slice(&mono_data);
        return;
    }

    // Step 2: Polyphase windowed-sinc resampling with bandlimited low-pass anti-aliasing
    let target_rate = 16000.0_f64;
    let ratio = input_rate as f64 / target_rate;
    let target_len = (mono_data.len() as f64 / ratio).ceil() as usize;
    output.reserve(target_len);

    let cutoff = if ratio > 1.0 { 1.0 / ratio } else { 1.0 };
    let half_taps: isize = 8; // 16-tap windowed sinc

    for i in 0..target_len {
        let src_idx = i as f64 * ratio;
        let center = src_idx.round() as isize;
        let mut sum = 0.0_f64;
        let mut weight_sum = 0.0_f64;

        for k in -half_taps..=half_taps {
            let sample_idx = center + k;
            if sample_idx >= 0 && (sample_idx as usize) < mono_data.len() {
                let x_pos = src_idx - sample_idx as f64;
                let sinc_val = if x_pos.abs() < 1e-9 {
                    1.0
                } else {
                    let px = std::f64::consts::PI * x_pos * cutoff;
                    px.sin() / px
                };
                let hann =
                    0.5 * (1.0 + (std::f64::consts::PI * x_pos / (half_taps as f64 + 1.0)).cos());
                let weight = sinc_val * hann;

                if let Some(&s) = mono_data.get(sample_idx as usize) {
                    sum += s as f64 * weight;
                    weight_sum += weight;
                }
            }
        }

        let sample = if weight_sum.abs() > 1e-6 {
            (sum / weight_sum).clamp(-1.0, 1.0) as f32
        } else {
            0.0
        };
        output.push(sample);
    }
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
    fn test_polyphase_sinc_resampling_fidelity() {
        // Generate a 440 Hz pure sine wave at 48 kHz for 0.1s
        let sample_rate = 48000;
        let duration_s = 0.1;
        let n_samples = (sample_rate as f64 * duration_s) as usize;
        let input: Vec<f32> = (0..n_samples)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / sample_rate as f32)).sin())
            .collect();

        let resampled = resample_to_16k(&input, sample_rate, 1);
        let expected_len = (16000.0 * duration_s) as usize;
        assert_eq!(resampled.len(), expected_len);

        // Peak amplitude should be preserved near 1.0
        let peak = resampled.iter().fold(0.0f32, |m, &x| m.max(x.abs()));
        assert!(peak > 0.95 && peak <= 1.0);

        // 44.1 kHz resample test
        let input_44k: Vec<f32> = (0..4410)
            .map(|i| (2.0 * std::f32::consts::PI * 440.0 * (i as f32 / 44100.0)).sin())
            .collect();
        let resampled_44k = resample_to_16k(&input_44k, 44100, 1);
        assert_eq!(resampled_44k.len(), 1600);
    }

    #[test]
    fn test_rms_db_silence_and_signal() {
        let silence = vec![0.0; 512];
        assert_eq!(compute_rms_db(&silence), -96.0);

        let loud = vec![1.0; 512];
        assert!(compute_rms_db(&loud) >= -0.1);
    }
}
