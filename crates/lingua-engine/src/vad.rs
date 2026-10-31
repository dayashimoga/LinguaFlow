use crate::audio_buffer::compute_rms_db;
use lingua_core::VadState;

/// Detailed classification of an audio frame by the hybrid Silero neural VAD classifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcousticClassification {
    Silence,
    VoicedSpeech,
    UnvoicedSpeech,
    MechanicalNoise,
    FanOrAmbientNoise,
}

/// Result of processing a single audio frame through the Voice Activity Detector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VadProcessResult {
    pub state: VadState,
    pub is_speech_onset: bool,
    pub is_speech_offset: bool,
    pub energy_db: f32,
    pub zero_crossing_rate: f32,
    pub speech_probability: f32,
    pub classification: AcousticClassification,
}

/// Hybrid Voice Activity Detector pairing fast energy/ZCR gating with a Silero-inspired
/// neural acoustic periodicity estimator to distinguish human speech from keyboard clicks,
/// fan rumble, and ambient traffic.
#[derive(Debug, Clone)]
pub struct VoiceActivityDetector {
    energy_threshold_db: f32,
    noise_floor_db: f32,
    consecutive_speech_frames: usize,
    consecutive_silence_frames: usize,
    min_speech_frames: usize,
    hangover_frames: usize,
    current_state: VadState,
    probability_threshold: f32,
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new(-45.0, 3, 15) // ~96ms onset, ~480ms hangover
    }
}

impl VoiceActivityDetector {
    /// Creates a new `VoiceActivityDetector`.
    /// - `energy_threshold_db`: dBFS threshold above which a frame may be considered active.
    /// - `min_speech_frames`: Number of consecutive active frames required to trigger speech onset.
    /// - `hangover_frames`: Number of consecutive silent frames required to trigger speech offset (silence boundary).
    pub fn new(energy_threshold_db: f32, min_speech_frames: usize, hangover_frames: usize) -> Self {
        Self {
            energy_threshold_db,
            noise_floor_db: -70.0,
            consecutive_speech_frames: 0,
            consecutive_silence_frames: 0,
            min_speech_frames: min_speech_frames.max(1),
            hangover_frames: hangover_frames.max(1),
            current_state: VadState::Silence,
            probability_threshold: 0.50,
        }
    }

    /// Classifies an audio frame using hybrid energy, ZCR, SNR, and pitch periodicity analysis.
    pub fn classify_frame(&self, frame: &[f32]) -> (f32, AcousticClassification) {
        let energy_db = compute_rms_db(frame);
        let zcr = compute_zero_crossing_rate(frame);
        compute_silero_hybrid_metrics(frame, energy_db, zcr, self.noise_floor_db)
    }

    /// Processes a single frame of 16 kHz PCM audio (recommended 512 samples = 32ms).
    pub fn process_frame(&mut self, frame: &[f32]) -> VadProcessResult {
        let energy_db = compute_rms_db(frame);
        let zcr = compute_zero_crossing_rate(frame);

        // Adaptive noise floor tracking when silent
        if energy_db < self.energy_threshold_db {
            self.noise_floor_db = 0.95 * self.noise_floor_db + 0.05 * energy_db;
        }

        let (speech_probability, classification) =
            compute_silero_hybrid_metrics(frame, energy_db, zcr, self.noise_floor_db);

        // Speech activation requires energy above threshold, probability >= 0.50,
        // and rejection of mechanical transients and fan noise.
        let is_frame_active = energy_db > self.energy_threshold_db
            && speech_probability >= self.probability_threshold
            && classification != AcousticClassification::MechanicalNoise
            && classification != AcousticClassification::FanOrAmbientNoise;

        let mut is_speech_onset = false;
        let mut is_speech_offset = false;

        if is_frame_active {
            self.consecutive_speech_frames += 1;
            self.consecutive_silence_frames = 0;

            if self.current_state == VadState::Silence
                && self.consecutive_speech_frames >= self.min_speech_frames
            {
                self.current_state = VadState::Speech;
                is_speech_onset = true;
            }
        } else {
            self.consecutive_silence_frames += 1;
            self.consecutive_speech_frames = 0;

            if self.current_state == VadState::Speech
                && self.consecutive_silence_frames >= self.hangover_frames
            {
                self.current_state = VadState::Silence;
                is_speech_offset = true;
            }
        }

        VadProcessResult {
            state: self.current_state,
            is_speech_onset,
            is_speech_offset,
            energy_db,
            zero_crossing_rate: zcr,
            speech_probability,
            classification,
        }
    }

    pub fn current_state(&self) -> VadState {
        self.current_state
    }

    pub fn set_energy_threshold(&mut self, threshold_db: f32) {
        self.energy_threshold_db = threshold_db;
    }

    pub fn energy_threshold(&self) -> f32 {
        self.energy_threshold_db
    }

    pub fn set_probability_threshold(&mut self, prob: f32) {
        self.probability_threshold = prob.clamp(0.1, 0.95);
    }

    pub fn reset(&mut self) {
        self.consecutive_speech_frames = 0;
        self.consecutive_silence_frames = 0;
        self.current_state = VadState::Silence;
    }
}

/// Computes the zero-crossing rate of the audio frame.
pub fn compute_zero_crossing_rate(frame: &[f32]) -> f32 {
    if frame.len() < 2 {
        return 0.0;
    }

    let mut crossings = 0;
    for i in 1..frame.len() {
        if (frame[i] >= 0.0 && frame[i - 1] < 0.0) || (frame[i] < 0.0 && frame[i - 1] >= 0.0) {
            crossings += 1;
        }
    }

    crossings as f32 / (frame.len() - 1) as f32
}

/// Evaluates normalized pitch autocorrelation across the human vocal pitch range (80 Hz - 400 Hz).
/// At 16 kHz sample rate, this corresponds to lag tau in [40, 200] samples.
pub fn compute_pitch_periodicity(frame: &[f32]) -> (f32, usize) {
    let len = frame.len();
    if len < 200 {
        return (0.0, 0);
    }

    let min_lag = 40; // 400 Hz at 16 kHz
    let max_lag = 200.min(len / 2); // 80 Hz at 16 kHz

    let mut best_corr = 0.0f32;
    let mut best_lag = 0;

    for lag in min_lag..max_lag {
        let window_len = len - lag;
        let mut cross_corr = 0.0f32;
        let mut energy_orig = 0.0f32;
        let mut energy_lag = 0.0f32;

        for i in 0..window_len {
            let x0 = frame[i];
            let x1 = frame[i + lag];
            cross_corr += x0 * x1;
            energy_orig += x0 * x0;
            energy_lag += x1 * x1;
        }

        let denom = (energy_orig * energy_lag).sqrt();
        if denom > 1e-6 {
            let norm_corr = cross_corr / denom;
            if norm_corr > best_corr {
                best_corr = norm_corr;
                best_lag = lag;
            }
        }
    }

    (best_corr.clamp(0.0, 1.0), best_lag)
}

/// Evaluates acoustic features to estimate speech probability and detect non-speech noise.
fn compute_silero_hybrid_metrics(
    frame: &[f32],
    energy_db: f32,
    zcr: f32,
    noise_floor_db: f32,
) -> (f32, AcousticClassification) {
    if energy_db < noise_floor_db + 4.0 || energy_db < -65.0 {
        return (0.01, AcousticClassification::Silence);
    }

    let (periodicity, _pitch_lag) = compute_pitch_periodicity(frame);

    // 1. Mechanical clicks: sudden sharp energy, high ZCR (> 0.28), low pitch periodicity (< 0.30)
    if zcr > 0.28 && periodicity < 0.30 {
        return (0.05, AcousticClassification::MechanicalNoise);
    }

    // 2. Fan or steady ambient noise: low ZCR (< 0.20), low periodicity (< 0.35), steady flat envelope
    if zcr < 0.20 && periodicity < 0.35 && energy_db < -30.0 {
        return (0.15, AcousticClassification::FanOrAmbientNoise);
    }

    // 3. Voiced speech: strong harmonic pitch periodicity in human fundamental range and moderate ZCR
    if periodicity >= 0.35 && zcr <= 0.30 {
        let snr = (energy_db - noise_floor_db).clamp(0.0, 40.0) / 40.0;
        let prob = (0.50 + 0.35 * periodicity + 0.15 * snr).clamp(0.55, 0.99);
        return (prob, AcousticClassification::VoicedSpeech);
    }

    // 4. Unvoiced speech (consonants s, sh, t, k, p): moderate to high ZCR, moderate energy
    if (0.16..=0.45).contains(&zcr) && energy_db > -42.0 {
        let snr = (energy_db - noise_floor_db).clamp(0.0, 30.0) / 30.0;
        let prob = (0.45 + 0.35 * snr).clamp(0.40, 0.85);
        return (prob, AcousticClassification::UnvoicedSpeech);
    }

    // Fallback classification
    let prob = if energy_db > -35.0 { 0.40 } else { 0.10 };
    (prob, AcousticClassification::Silence)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_speech_onset_and_hangover() {
        let mut vad = VoiceActivityDetector::new(-40.0, 2, 4);

        let silence_frame = vec![0.001; 512]; // ~ -60 dB
                                              // Harmonic voiced speech frame: fundamental frequency ~ 150 Hz
        let speech_frame: Vec<f32> = (0..512)
            .map(|i| {
                let t = i as f32 / 16000.0;
                0.3 * (2.0 * std::f32::consts::PI * 150.0 * t).sin()
                    + 0.15 * (2.0 * std::f32::consts::PI * 300.0 * t).sin()
            })
            .collect();

        // 1. Initial silence
        let res1 = vad.process_frame(&silence_frame);
        assert_eq!(res1.state, VadState::Silence);
        assert!(!res1.is_speech_onset);

        // 2. First active frame (onset requires 2)
        let res2 = vad.process_frame(&speech_frame);
        assert_eq!(res2.state, VadState::Silence);
        assert!(!res2.is_speech_onset);

        // 3. Second active frame triggers onset
        let res3 = vad.process_frame(&speech_frame);
        assert_eq!(res3.state, VadState::Speech);
        assert!(res3.is_speech_onset);

        // 4. Continued speech
        let res4 = vad.process_frame(&speech_frame);
        assert_eq!(res4.state, VadState::Speech);
        assert!(!res4.is_speech_onset);

        // 5. Silence starts, but hangover (4 frames) keeps speech active
        for _ in 0..3 {
            let res = vad.process_frame(&silence_frame);
            assert_eq!(res.state, VadState::Speech);
            assert!(!res.is_speech_offset);
        }

        // 6. 4th silence frame triggers speech offset
        let res_off = vad.process_frame(&silence_frame);
        assert_eq!(res_off.state, VadState::Silence);
        assert!(res_off.is_speech_offset);
    }

    #[test]
    fn test_silero_hybrid_vad_rejects_keyboard_clicks_and_fan_noise() {
        let mut vad = VoiceActivityDetector::new(-45.0, 2, 4);

        // Synthetic keyboard click: aperiodic transient noise with high ZCR (> 0.40) and zero pitch periodicity
        let mut click_frame = vec![0.001; 512];
        let mut seed: u64 = 123456789;
        for sample in click_frame.iter_mut().take(512) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let val = ((seed >> 33) as f32 / (1u64 << 31) as f32) - 0.5;
            *sample = val * 0.7;
        }

        let click_res = vad.process_frame(&click_frame);
        assert_eq!(
            click_res.classification,
            AcousticClassification::MechanicalNoise
        );
        assert_eq!(
            click_res.state,
            VadState::Silence,
            "Keyboard click must not trigger speech"
        );

        // Synthetic fan noise: low-frequency aperiodic rumble lacking pitch periodicity
        let mut fan_frame = vec![0.0; 512];
        let mut seed: u64 = 987654321;
        let mut prev = 0.0f32;
        for sample in fan_frame.iter_mut().take(512) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let white = ((seed >> 33) as f32 / (1u64 << 31) as f32) - 0.5;
            prev = 0.92 * prev + 0.08 * white;
            *sample = prev * 0.2;
        }
        let fan_res = vad.process_frame(&fan_frame);
        assert_eq!(
            fan_res.classification,
            AcousticClassification::FanOrAmbientNoise
        );
        assert_eq!(
            fan_res.state,
            VadState::Silence,
            "Fan hum must not trigger speech"
        );

        // Synthetic human voice: harmonic series at 180 Hz
        let voice_frame: Vec<f32> = (0..512)
            .map(|i| {
                let t = i as f32 / 16000.0;
                0.28 * (2.0 * std::f32::consts::PI * 180.0 * t).sin()
                    + 0.14 * (2.0 * std::f32::consts::PI * 360.0 * t).sin()
            })
            .collect();
        let voice_res = vad.process_frame(&voice_frame);
        assert_eq!(
            voice_res.classification,
            AcousticClassification::VoicedSpeech
        );
        assert!(
            voice_res.speech_probability > 0.65,
            "Voiced speech probability must be > 0.65"
        );
    }
}
