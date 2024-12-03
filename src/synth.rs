use std::f32::consts::PI;
use crate::random::Random;
use crate::filter::LowpassFilter;

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
    sample_rate: i32,
    phase: f32,
    frequency_drift: f32,
    volume_drift: f32,
    vibrato_phase: f32,
    vibrato_frequency_drift: f32,
    vibrato_amplitude_drift: f32,
    random: Random,
    noise_filter: LowpassFilter,
    params_valid: bool,
    alpha: f32,
    epsilon: f32,
    e0: f32,
    ta: f32,
    tp: f32,
    te: f32,
    shift: f32,
    step: i64
}

impl Glottis {
    pub fn new(sample_rate: i32) -> Self {
        let mut random = Random::new();
        Self {
            frequency: 220.0,
            rd: 1.7,
            noise: 0.01,
            frequency_drift_amplitude: 0.003,
            volume_drift_amplitude: 0.1,
            vibrato_frequency: 5.0,
            vibrato_amplitude: 0.01,
            vibrato_frequency_drift_amplitude: 0.05,
            vibrato_amplitude_drift_amplitude: 0.2,
            sample_rate: sample_rate,
            phase: random.get_uniform(),
            frequency_drift: 0.0,
            volume_drift: 0.0,
            vibrato_phase: random.get_uniform(),
            vibrato_frequency_drift: 0.0,
            vibrato_amplitude_drift: 0.0,
            random: random,
            noise_filter: LowpassFilter::new(sample_rate, 2000.0),
            params_valid: false,
            alpha: 0.0,
            epsilon: 0.0,
            e0: 0.0,
            ta: 0.0,
            tp: 0.0,
            te: 0.0,
            shift: 0.0,
            step: 0
        }
    }

    pub fn generate(&mut self) -> f32 {
        // If rd has changed, recalculate all the parameters that depend on it.

        if !self.params_valid {
            let ra = 0.048*self.rd - 0.01;
            let rk = 0.118*self.rd + 0.224;
            let rg = 0.25*rk / (0.11*self.rd / (0.5+1.2*rk) - ra);
            self.ta = ra;
            self.tp = 0.5/rg;
            self.te = self.tp*(1.0+rk);
            self.epsilon = 1.0/self.ta;
            self.alpha = 8.08*(-1.07*self.rd).exp();
            self.e0 = 1.0/((self.alpha*self.te).exp()*(PI*self.te/self.tp).sin());
            self.shift = (-self.epsilon*(1.0-self.te)).exp();
            self.params_valid = true;
        }

        // Randomly vary aspects of the output to make it sound more natural.

        if self.step % 1000 == 0 {
            self.frequency_drift = 0.99*self.frequency_drift + 0.1*self.random.get_normal();
            self.volume_drift = 0.99*self.volume_drift + 0.1*self.random.get_normal();
            self.vibrato_frequency_drift = 0.99*self.vibrato_frequency_drift + 0.1*self.random.get_normal();
            self.vibrato_amplitude_drift = 0.99*self.vibrato_amplitude_drift + 0.1*self.random.get_normal();
        }
        self.step += 1;

        // Compute the instantaneous frequency and update the current phase.
        // This depends on the primary frequency of the note, vibrato, and
        // random drift.

        let vibrato_freq = self.vibrato_frequency * (1.0+self.vibrato_frequency_drift_amplitude*self.vibrato_frequency_drift);
        let vibrato_amplitude = self.vibrato_amplitude * (1.0+self.vibrato_amplitude_drift_amplitude*self.vibrato_amplitude_drift);
        let vibrato_offset = vibrato_freq / self.sample_rate as f32;
        self.vibrato_phase = (self.vibrato_phase+vibrato_offset) % 1.0;
        let freq = self.frequency * (1.0+self.frequency_drift_amplitude*self.frequency_drift) * (1.0+self.vibrato_amplitude*((2.0*PI*self.vibrato_phase).sin()));
        let offset = freq / self.sample_rate as f32;
        self.phase = (self.phase+offset) % 1.0;
        let t = self.phase;

        // Add noise consisting of regular peaks plus a uniform background.

        let x = (t-0.5-(t-0.5).round()).abs();
        let window = if x < 0.25 {0.5+0.5*(4.0*PI*x).cos()} else {0.0};
        let noise = self.noise_filter.process(self.noise*(0.2+window)*self.random.get_normal());

        // Compute the output.

        let volume = 1.0 + self.volume_drift_amplitude*self.volume_drift;
        if t < self.te {
            return volume*(noise + self.e0*(self.alpha*t).exp()*(PI*t/self.tp).sin());
        }
        volume*(noise + ((-self.epsilon*(t-self.te)).exp() - self.shift)/(self.epsilon*self.ta))
    }
}

pub struct Waveguide {
    area: Vec<f32>,
    k: Vec<f32>,
    right: Vec<f32>,
    left: Vec<f32>
}

impl Waveguide {
    pub fn new() -> Self {
        let mut waveguide = Self {
            area: vec![0.0; 44],
            k: vec![0.0; 44],
            right: vec![0.0; 44],
            left: vec![0.0; 44]
        };
        waveguide.compute_reflections();
        waveguide
    }

    pub fn set_shape(&mut self, shape: &Vec<f32>) {
        self.area = shape.to_vec();
        self.compute_reflections();
    }

    pub fn compute_reflections(&mut self) {
        let n = self.k.len();
        for i in 0..n-1 {
            if self.area[i+1] == 0.0 {
                self.k[i] = 1.0;
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

pub struct Voice {
    first: bool,
    glottis: Glottis,
    vocal: Waveguide,
    nasal: Waveguide,
    nasal_coupling: f32
}

impl Voice {
    pub fn new(sample_rate: i32) -> Self {
        Voice {
            first: true,
            glottis: Glottis::new(sample_rate),
            vocal: Waveguide::new(),
            nasal: Waveguide::new(),
            nasal_coupling: 0.0
        }
    }

    pub fn set_vocal_shape(&mut self, shape: &Vec<f32>, nasal_coupling: f32) {
        self.vocal.set_shape(shape);
        self.nasal_coupling = nasal_coupling;
    }

    pub fn generate(&mut self) -> f32 {
        let excitation = self.glottis.generate();
        let n = self.vocal.right.len();
        for j in 0..2 {
            let right = self.vocal.right.clone();
            let left = self.vocal.left.clone();
            let right_output = &mut self.vocal.right;
            let left_output = &mut self.vocal.left;
            let k = &self.vocal.k;
            right_output[0] = excitation + left[0];
            for i in 1..n {
                let w = k[i] * (right[i-1]+left[i]);
                right_output[i] = right[i-1] - w;
                left_output[i-1] = left[i] + w;
            }
        }
        self.vocal.right[n-1]
    }
}