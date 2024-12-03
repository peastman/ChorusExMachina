use std::f32::consts::PI;

/// An IIR lowpass filter.
pub struct LowpassFilter {
    alpha: f32,
    y: f32
}

impl LowpassFilter {
    pub fn new(sampling_rate: i32, cutoff: f32) -> Self {
        let rc = 1.0/(2.0*PI*cutoff);
        let dt = 1.0/sampling_rate as f32;
        let alpha = dt/(rc+dt);
        Self {
            alpha: alpha,
            y: 0.0
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        self.y += self.alpha*(x-self.y);
        self.y
    }
}

pub struct HighpassFilter {
    alpha: f32,
    x: f32,
    y: f32
}

/// An IIR highpass filter.
impl HighpassFilter {
    pub fn new(sampling_rate: i32, cutoff: f32) -> Self {
        let rc = 1.0/(2.0*PI*cutoff);
        let dt = 1.0/sampling_rate as f32;
        let alpha = rc/(rc+dt);
        Self {
            alpha: alpha,
            x: 0.0,
            y: 0.0
        }
    }

    pub fn process(&mut self, x: f32) -> f32 {
        self.y = self.alpha * (self.y+x-self.x);
        self.x = x;
        self.y
    }
}