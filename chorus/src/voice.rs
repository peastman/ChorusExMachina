use std::f32::consts::PI;
use crate::random::Random;
use crate::filter::{Filter, LowpassFilter};
use crate::VoicePart;
use crate::SAMPLE_RATE;

/// This implements the glottal excitation of the source-filter model.  It consists of
/// a Liljencrants-Fant (LF) model plus pulsed noise.  To improve realism, it adds
/// random fluctuations to several aspects of the output: frequency, amplitude,
/// vibrato frequency, and vibrato amplitude.
pub struct Glottis {
    pub frequency: f32,
    pub rd: f32,
    pub noise: f32,
    pub frequency_drift_amplitude: f32,
    pub volume_drift_amplitude: f32,
    pub vibrato_frequency: f32,
    pub vibrato_amplitude: f32,
    pub vibrato_frequency_drift_amplitude: f32,
    pub vibrato_amplitude_drift_amplitude: f32,
    phase: f32,
    frequency_drift: f32,
    volume_drift: f32,
    vibrato_phase: f32,
    vibrato_frequency_drift: f32,
    vibrato_amplitude_drift: f32,
    random: Random,
    noise_filter: LowpassFilter,
    last_rd: f32,
    alpha: f32,
    epsilon: f32,
    e0: f32,
    ta: f32,
    tp: f32,
    te: f32,
    shift: f32
}

impl Glottis {
    pub fn new() -> Self {
        let mut random = Random::new();
        Self {
            frequency: 220.0,
            rd: 1.7,
            noise: 0.01,
            frequency_drift_amplitude: 0.003,
            volume_drift_amplitude: 0.1,
            vibrato_frequency: 5.0,
            vibrato_amplitude: 0.02,
            vibrato_frequency_drift_amplitude: 0.05,
            vibrato_amplitude_drift_amplitude: 0.4,
            phase: random.get_uniform(),
            frequency_drift: 0.0,
            volume_drift: 0.0,
            vibrato_phase: random.get_uniform(),
            vibrato_frequency_drift: 0.0,
            vibrato_amplitude_drift: 0.0,
            random: random,
            noise_filter: LowpassFilter::new(2000.0),
            last_rd: 0.0,
            alpha: 0.0,
            epsilon: 0.0,
            e0: 0.0,
            ta: 0.0,
            tp: 0.0,
            te: 0.0,
            shift: 0.0
        }
    }

    pub fn generate(&mut self, step: i64) -> f32 {
        // If rd has changed, recalculate all the parameters that depend on it.

        if self.rd != self.last_rd {
            let ra = 0.048*self.rd - 0.01;
            let rk = 0.118*self.rd + 0.224;
            let rg = 0.25*rk / (0.11*self.rd / (0.5+1.2*rk) - ra);
            self.ta = ra;
            self.tp = 0.5/rg;
            self.te = self.tp*(1.0+rk);

            // According to the LF model, we're supposed to iteratively solve nonlinear equations
            // to determine alpha and epsilon.  That is slow.  The following approximations are
            // very fast and produce good results.

            self.epsilon = 1.0/self.ta;
            self.alpha = 8.08*(-1.07*self.rd).exp();
            self.e0 = 1.0/((self.alpha*self.te).exp()*(PI*self.te/self.tp).sin());
            self.shift = (-self.epsilon*(1.0-self.te)).exp();
            self.last_rd = self.rd;
        }

        // Randomly vary aspects of the output to make it sound more natural.

        if step % 1000 == 0 {
            self.frequency_drift = 0.99*self.frequency_drift + 0.1*self.random.get_normal();
            self.volume_drift = 0.99*self.volume_drift + 0.1*self.random.get_normal();
            self.vibrato_frequency_drift = 0.99*self.vibrato_frequency_drift + 0.1*self.random.get_normal();
            self.vibrato_amplitude_drift = 0.99*self.vibrato_amplitude_drift + 0.1*self.random.get_normal();
        }

        // Compute the instantaneous frequency and update the current phase.
        // This depends on the primary frequency of the note, vibrato, and
        // random drift.

        let vibrato_freq = self.vibrato_frequency * (1.0+self.vibrato_frequency_drift_amplitude*self.vibrato_frequency_drift);
        let vibrato_amplitude = self.vibrato_amplitude * (1.0+self.vibrato_amplitude_drift_amplitude*self.vibrato_amplitude_drift);
        let vibrato_offset = vibrato_freq / SAMPLE_RATE as f32;
        self.vibrato_phase = (self.vibrato_phase+vibrato_offset) % 1.0;
        let freq = self.frequency * (1.0+self.frequency_drift_amplitude*self.frequency_drift) * (1.0+vibrato_amplitude*((2.0*PI*self.vibrato_phase).sin()));
        let offset = freq / SAMPLE_RATE as f32;
        self.phase = (self.phase+offset) % 1.0;
        let t = self.phase;

        // Add noise consisting of regular peaks plus a uniform background.

        let x = (t-0.5-(t-0.5).round()).abs();
        let window = if x < 0.25 {0.5+0.5*(4.0*PI*x).cos()} else {0.0};
        let noise = self.noise_filter.process(self.noise*(0.2+window)*(2.0*self.random.get_uniform()-1.0));

        // Compute the output.

        let volume = 1.0 + self.volume_drift_amplitude*self.volume_drift;
        if t < self.te {
            return volume*(noise + self.e0*(self.alpha*t).exp()*(PI*t/self.tp).sin());
        }
        volume*(noise + ((-self.epsilon*(t-self.te)).exp() - self.shift)/(self.epsilon*self.ta))
    }
}

/// A 1D waveguide along which an audio signal can propagate.  A Voice uses two of these,
/// one for the vocal tract and one for the nasal cavity.
pub struct Waveguide {
    area: Vec<f32>,
    k: Vec<f32>,
    right: Vec<f32>,
    left: Vec<f32>
}

impl Waveguide {
    pub fn new(length: usize) -> Self {
        let mut waveguide = Self {
            area: vec![0.0; length],
            k: vec![0.0; length],
            right: vec![0.0; length],
            left: vec![0.0; length]
        };
        waveguide.compute_reflections();
        waveguide
    }

    /// Set the shape of the waveguide, specified as the area of each segment.
    pub fn set_shape(&mut self, shape: &Vec<f32>) {
        self.area = shape.to_vec();
        self.compute_reflections();
    }

    /// Compute the reflection coefficients for the segments.  This is called automatically
    /// whenever the shape changes.
    pub fn compute_reflections(&mut self) {
        let n = self.k.len();
        for i in 0..n-1 {
            if self.area[i+1] == 0.0 {
                self.k[i] = 0.98;
                self.right[i] = 0.0;
                self.left[i] = 0.0;
            }
            else {
                self.k[i+1] = (self.area[i]-self.area[i+1]) / (self.area[i]+self.area[i+1]);
            }
        }
        self.k[n-1] = -0.85;
    }
}

/// This struct combines a glottal source and two waveguides to form the complete synthesis model.
/// In addition, consonants can be synthesized by injecting extra noise at an arbitrary point in
/// the vocal tract.
pub struct Voice {
    glottis: Glottis,
    vocal: Waveguide,
    nasal: Waveguide,
    volume: f32,
    nasal_coupling: f32,
    coupling_position: usize,
    nasal_off_after_step: i64
}

impl Voice {
    pub fn new(voice_part: VoicePart) -> Self {
        let vocal_length;
        let coupling_position;
        let vibrato_frequency;
        let nasal_shape;
        match voice_part {
            VoicePart::Soprano => {
                vocal_length = 42;
                coupling_position = 22;
                vibrato_frequency = 6.3;
                nasal_shape = vec![1.52, 1.71, 2.08, 2.78, 3.53, 4.28, 4.33, 2.89, 2.49, 2.42, 2.13, 1.95, 1.83, 1.74, 1.41, 0.878, 0.769, 1.15, 1.12, 1.09, 1.06, 0.672];
            }
            VoicePart::Alto => {
                vocal_length = 45;
                coupling_position = 23;
                vibrato_frequency = 5.3;
                nasal_shape = vec![1.52, 1.7, 2.04, 2.68, 3.38, 4.18, 4.4, 3.4, 2.45, 2.46, 2.28, 2.0, 1.88, 1.8, 1.65, 1.24, 0.735, 0.873, 1.17, 1.09, 1.1, 1.03, 0.667];
            }
            VoicePart::Tenor => {
                vocal_length = 48;
                coupling_position = 24;
                vibrato_frequency = 5.3;
                nasal_shape = vec![1.52, 1.7, 2.04, 2.68, 3.38, 4.18, 4.4, 3.4, 2.45, 2.46, 2.28, 2.0, 1.88, 1.8, 1.65, 1.24, 0.735, 0.873, 1.17, 1.09, 1.1, 1.03, 0.667];
            }
            VoicePart::Bass => {
                vocal_length = 50;
                coupling_position = 25;
                vibrato_frequency = 5.2;
                nasal_shape = vec![1.52, 1.69, 1.99, 2.59, 3.3, 4.02, 4.44, 3.77, 2.57, 2.49, 2.38, 2.09, 1.94, 1.83, 1.76, 1.52, 1.05, 0.699, 0.972, 1.18, 1.07, 1.12, 1.01, 0.663];
            }
        }
        let mut voice = Voice {
            glottis: Glottis::new(),
            vocal: Waveguide::new(vocal_length),
            nasal: Waveguide::new(nasal_shape.len()),
            volume: 1.0,
            nasal_coupling: 0.0,
            coupling_position: coupling_position,
            nasal_off_after_step: 0
        };
        voice.nasal.set_shape(&nasal_shape);
        voice.glottis.vibrato_frequency = vibrato_frequency;
        voice
    }

    /// Set the volume of the glottal excitation (between 0.0 and 1.0).
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume;
    }

    /// Set the shape of the vocal tract, specified as a vector of cross-sectional areas
    /// for each segment.  Also specified is the degree of coupling between the vocal
    /// tract and nasal cavity.  This should be 0.5 for nasal sounds like m and n, 0.0
    /// for most others.
    pub fn set_vocal_shape(&mut self, shape: &Vec<f32>, nasal_coupling: f32) {
        self.vocal.set_shape(shape);
        self.nasal_coupling = nasal_coupling;
    }

    /// Set the frequency of the glottal excitation (in Hz).
    pub fn set_frequency(&mut self, frequency: f32) {
        self.glottis.frequency = frequency;
    }

    /// Set the Rd parameter for the LF model.  This controls the overall intensity of the sound.
    /// Typical values are in the range of about 1.0 to 2.5.  Lower values produce a pressed
    /// sound, while higher values produce a more relaxed sound.
    pub fn set_rd(&mut self, rd: f32) {
        self.glottis.rd = rd;
    }

    /// Set the amplitude of the glottal noise.  Larger values produce a more breathy sound.
    pub fn set_noise(&mut self, noise: f32) {
        self.glottis.noise = noise;
    }

    /// Get the frequency of vibrato.
    pub fn get_vibrato_frequency(&self) -> f32 {
        self.glottis.vibrato_frequency
    }

    /// Set the frequency of vibrato.
    pub fn set_vibrato_frequency(&mut self, frequency: f32) {
        self.glottis.vibrato_frequency = frequency;
    }

    /// Set the amplitude of vibrato.
    pub fn set_vibrato_amplitude(&mut self, amplitude: f32) {
        self.glottis.vibrato_amplitude = amplitude;
    }

    /// Generate the next audio sample.  Arguments are the current sample index, the noise signal
    /// to inject into the vocal tract (to simulate consonants), and the position at which to
    /// inject it.
    pub fn generate(&mut self, step: i64, noise: f32, noise_position: usize) -> f32 {
        if self.vocal.area[noise_position] > 0.0 {
            self.vocal.right[noise_position] += noise;
            self.vocal.left[noise_position] += noise;
        }
        let excitation = self.volume*self.glottis.generate(step);
        let n = self.vocal.right.len();
        let nasal_n = self.nasal.right.len();
        let damping = 0.995;
        for _substep in 0..2 {
            // Propagate waves in the vocal tract.

            let right = self.vocal.right.clone();
            let left = self.vocal.left.clone();
            let right_output = &mut self.vocal.right;
            let left_output = &mut self.vocal.left;
            let k = &self.vocal.k;
            right_output[0] = excitation + left[0];
            for i in 1..n {
                let w = k[i] * (right[i-1]+left[i]);
                right_output[i] = damping*(right[i-1] - w);
                left_output[i-1] = damping*(left[i] + w);
            }
            if self.nasal_coupling > 0.0 {
                self.nasal_off_after_step = step+500;
            }
            if step < self.nasal_off_after_step {
                // Propagate waves in the nasal cavity.

                let damping = 0.99;
                let nasal_right = self.nasal.right.clone();
                let nasal_left = self.nasal.left.clone();
                let nasal_right_output = &mut self.nasal.right;
                let nasal_left_output = &mut self.nasal.left;
                let nasal_k = &self.nasal.k;
                nasal_right_output[0] = nasal_left[0];
                for i in 1..nasal_n {
                    let w = nasal_k[i] * (nasal_right[i-1]+nasal_left[i]);
                    nasal_right_output[i] = damping*(nasal_right[i-1] - w);
                    nasal_left_output[i-1] = damping*(nasal_left[i] + w);
                }

                // Connect them together.

                if self.nasal_coupling != 0.0 {
                    let pos = self.coupling_position;
                    let w1 = self.nasal_coupling;
                    let w2 = 1.0-self.nasal_coupling;
                    nasal_right_output[0] = w2*nasal_right_output[0] + w1*right_output[pos];
                    nasal_left_output[0] = w2*nasal_left_output[0] + w1*left_output[pos];
                    right_output[pos] = w1*nasal_right_output[0] + w2*right_output[pos];
                    left_output[pos] = w1*nasal_left_output[0] + w2*left_output[pos];
                }
            }
        }
        self.vocal.right[n-1] + self.nasal.right[nasal_n-1]
    }
}