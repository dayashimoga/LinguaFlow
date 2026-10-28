use crate::audio_buffer::compute_rms_db;
use lingua_core::VadState;

/// Result of processing a single audio frame through the Voice Activity Detector.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VadProcessResult {
    pub state: VadState,
    pub is_speech_onset: bool,
    pub is_speech_offset: bool,
    pub energy_db: f32,
    pub zero_crossing_rate: f32,
}

/// Voice Activity Detector (VAD) evaluating energy levels, zero-crossing rates,
/// and silence hangover durations on 16 kHz PCM audio frames.
#[derive(Debug, Clone)]
pub struct VoiceActivityDetector {
    energy_threshold_db: f32,
    noise_floor_db: f32,
    consecutive_speech_frames: usize,
    consecutive_silence_frames: usize,
    min_speech_frames: usize,
    hangover_frames: usize,
    current_state: VadState,
}

impl Default for VoiceActivityDetector {
    fn default() -> Self {
        Self::new(-45.0, 3, 15) // ~96ms onset, ~480ms hangover
    }
}

impl VoiceActivityDetector {
    /// Creates a new `VoiceActivityDetector`.
    /// - `energy_threshold_db`: dBFS threshold above which a frame is considered active speech.
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
        }
    }

    /// Processes a single frame of 16 kHz PCM audio (recommended 512 samples = 32ms).
    pub fn process_frame(&mut self, frame: &[f32]) -> VadProcessResult {
        let energy_db = compute_rms_db(frame);
        let zcr = compute_zero_crossing_rate(frame);

        // Adaptive noise floor tracking when silent
        if energy_db < self.energy_threshold_db {
            self.noise_floor_db = 0.95 * self.noise_floor_db + 0.05 * energy_db;
        }

        let is_frame_active = energy_db > self.energy_threshold_db;

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

    pub fn reset(&mut self) {
        self.consecutive_speech_frames = 0;
        self.consecutive_silence_frames = 0;
        self.current_state = VadState::Silence;
    }
}

/// Computes the zero-crossing rate of the audio frame.
fn compute_zero_crossing_rate(frame: &[f32]) -> f32 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vad_speech_onset_and_hangover() {
        let mut vad = VoiceActivityDetector::new(-40.0, 2, 4);

        let silence_frame = vec![0.001; 512]; // ~ -60 dB
        let speech_frame: Vec<f32> = (0..512).map(|i| 0.3 * (i as f32 * 0.1).sin()).collect(); // ~ -13 dB

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
}
