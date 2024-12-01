use std::f32::consts::PI;
use crate::random::Random;

pub struct Glottis {
    pub frequency: f32,
    pub rd: f32,
    pub noise: f32,
    pub drift_amplitude: f32,
    pub volume_drift_amplitude: f32,
    pub vibrato_frequency: f32,
    pub vibrato_amplitude: f32,
    pub vibrato_frequency_drift_amplitude: f32,
    pub vibrato_amplitude_drift_amplitude: f32,
    sample_rate: i32,
    phase: f32,
    drift: f32,
    volume_drift: f32,
    vibrato_phase: f32,
    vibrato_frequency_drift: f32,
    vibrato_amplitude_drift: f32,
    random: Random,
    params_valid: bool,
    alpha: f32,
    epsilon: f32,
    ee: f32,
    ta: f32,
    tp: f32,
    te: f32,
    shift: f32,
    step: i64
}

impl Glottis {
    pub fn new(sample_rate: i32) -> Self {
        Self {
            frequency: 220.0,
            rd: 1.0,
            noise: 0.02,
            drift_amplitude: 0.003,
            volume_drift_amplitude: 0.2,
            vibrato_frequency: 5.0,
            vibrato_amplitude: 0.01,
            vibrato_frequency_drift_amplitude: 0.05,
            vibrato_amplitude_drift_amplitude: 0.2,
            sample_rate: sample_rate,
            phase: 0.0,
            drift: 0.0,
            volume_drift: 0.0,
            vibrato_phase: 0.0,
            vibrato_frequency_drift: 0.0,
            vibrato_amplitude_drift: 0.0,
            random: Random::new(),
            params_valid: false,
            alpha: 0.0,
            epsilon: 0.0,
            ee: 0.0,
            ta: 0.0,
            tp: 0.0,
            te: 0.0,
            shift: 0.0,
            step: 0
        }
    }

    pub fn generate(&mut self) -> f32 {
        if !self.params_valid {
            let ra = 0.048*self.rd - 0.01;
            let rk = 0.118*self.rd + 0.224;
            let rg = 0.25*rk / (0.11*self.rd / (0.5+1.2*rk) - ra);
            self.ta = ra;
            self.tp = 0.5/rg;
            self.te = self.tp*(1.0+rk);
            self.epsilon = 1.0/self.ta;
            self.alpha = 8.08*(-1.07*self.rd).exp();
            self.ee = (self.alpha*self.te).exp()*(PI*self.te/self.tp).sin();
            self.shift = (-self.epsilon*(1.0-self.te)).exp();
            self.params_valid = true;
        }

        if self.step % 10 == 0 {
            self.drift = 0.9999*self.drift + 0.01*self.random.get_normal();
            self.volume_drift = 0.9999*self.volume_drift + 0.01*self.random.get_normal();
        }
        if self.step % 1000 == 0 {
            self.vibrato_frequency_drift = 0.99*self.vibrato_frequency_drift + 0.1*self.random.get_normal();
            self.vibrato_amplitude_drift = 0.99*self.vibrato_amplitude_drift + 0.1*self.random.get_normal();
        }

        let vibrato_freq = self.vibrato_frequency * (1.0+self.vibrato_frequency_drift_amplitude*self.vibrato_frequency_drift);
        let vibrato_amplitude = self.vibrato_amplitude * (1.0+self.vibrato_amplitude_drift_amplitude*self.vibrato_amplitude_drift);
        let vibrato_offset = vibrato_freq / self.sample_rate as f32;
        self.vibrato_phase = (self.vibrato_phase+vibrato_offset) % 1.0;
        let freq = self.frequency * (1.0+self.drift_amplitude*self.drift) * (1.0+self.vibrato_amplitude*((2.0*PI*self.vibrato_phase).sin()));
        // if self.step % 100 == 0 {
        //     println!("{freq}");
        // }
        let offset = freq / self.sample_rate as f32;
        self.phase = (self.phase+offset) % 1.0;

        let t = self.phase;

        let x = (t-t.round()).abs();
        let window = if x < 0.25 {0.5+0.5*(4.0*std::f32::consts::PI*x).cos()} else {0.0};
        let noise = self.noise*window*self.random.get_normal();

        self.step += 1;

        let volume = 1.0 + self.volume_drift_amplitude*self.volume_drift;
        if t < self.te {
            return volume*(noise + (self.alpha*t).exp()*(PI*t/self.tp).sin());
        }
        volume*(noise + (self.ee/(self.epsilon*self.ta))*((-self.epsilon*(t-self.te)).exp() - self.shift))
    }
}

pub struct Tract {
    area: Vec<f32>,
    k: Vec<f32>,
    right: Vec<f32>,
    left: Vec<f32>
}

impl Tract {
    pub fn new() -> Self {
        let mut tract = Self {
            area: vec![0.45, 0.26, 0.32, 0.33, 1.12, 0.63, 0.26, 0.23, 0.29, 0.40, 1.20, 1.62, 2.56, 2.86, 3.75, 5.09, 6.55, 6.27, 5.28, 3.87, 4.25, 4.69],
            // area: vec![0.33, 0.36, 0.68, 2.43, 2.66, 3.39, 3.78, 4.50, 4.68, 4.15, 3.51, 2.03, 1.38, 0.60, 0.32, 0.10, 0.25, 0.38, 0.36, 1.58, 2.01, 1.58],
            k: vec![0.0; 22],
            right: vec![0.0; 22],
            left: vec![0.0; 22]
        };
        tract.compute_reflections();
        tract
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
    vocal: Tract,
    nasal: Tract
}

impl Voice {
    pub fn new(sample_rate: i32) -> Self {
        Voice {
            first: true,
            glottis: Glottis::new(sample_rate),
            vocal: Tract::new(),
            nasal: Tract::new()
        }
    }

    pub fn generate(&mut self) -> f32 {
        let excitation = self.glottis.generate();
        self.first = false;
        let right = self.vocal.right.clone();
        let left = self.vocal.left.clone();
        let right_output = &mut self.vocal.right;
        let left_output = &mut self.vocal.left;
        let n = right.len();
        let k = &self.vocal.k;
        right_output[0] = excitation + left[0];
        for i in 1..n {
            let w = k[i] * (right[i-1]+left[i]);
            right_output[i] = right[i-1] - w;
            left_output[i-1] = left[i] + w;
        }
        right_output[n-1]
    }
}