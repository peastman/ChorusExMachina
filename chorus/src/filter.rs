use crate::SAMPLE_RATE;
use std::f32::consts::PI;

pub trait Filter {
    fn process(&mut self, x: f32) -> f32;
}

/// An IIR lowpass filter.
#[derive(Copy, Clone)]
pub struct LowpassFilter {
    alpha: f32,
    y: f32
}

impl LowpassFilter {
    pub fn new(cutoff: f32) -> Self {
        let rc = 1.0/(2.0*PI*cutoff);
        let dt = 1.0/SAMPLE_RATE as f32;
        let alpha = dt/(rc+dt);
        Self {
            alpha: alpha,
            y: 0.0
        }
    }
}

impl Filter for LowpassFilter {
    fn process(&mut self, x: f32) -> f32 {
        self.y += self.alpha*(x-self.y);
        self.y
    }
}

/// An IIR highpass filter.
#[derive(Copy, Clone)]
pub struct HighpassFilter {
    alpha: f32,
    x: f32,
    y: f32
}

impl HighpassFilter {
    pub fn new(cutoff: f32) -> Self {
        let rc = 1.0/(2.0*PI*cutoff);
        let dt = 1.0/SAMPLE_RATE as f32;
        let alpha = rc/(rc+dt);
        Self {
            alpha: alpha,
            x: 0.0,
            y: 0.0
        }
    }
}

impl Filter for HighpassFilter {
    fn process(&mut self, x: f32) -> f32 {
        self.y = self.alpha * (self.y+x-self.x);
        self.x = x;
        self.y
    }
}

/// An IIR bandpass filter.
#[derive(Copy, Clone)]
pub struct BandpassFilter {
    lowpass: LowpassFilter,
    highpass: HighpassFilter
}

impl BandpassFilter {
    pub fn new(low_cutoff: f32, high_cutoff: f32) -> Self {
        Self {
            lowpass: LowpassFilter::new(low_cutoff),
            highpass: HighpassFilter::new(high_cutoff)
        }
    }
}

impl Filter for BandpassFilter {
    fn process(&mut self, x: f32) -> f32 {
        self.highpass.process(self.lowpass.process(x))
    }
}

/// An IIR resonant filter.
#[derive(Copy, Clone)]
pub struct ResonantFilter {
    b1: f32,
    b2: f32,
    y1: f32,
    y2: f32
}

impl ResonantFilter {
    pub fn new(resonant_frequency: f32, bandwidth: f32) -> Self {
        let w = 2.0*PI*resonant_frequency/SAMPLE_RATE as f32;
        let r = 1.0 - PI*bandwidth/SAMPLE_RATE as f32;
        Self {
            b1: -2.0*r*w.cos(),
            b2: r*r,
            y1: 0.0,
            y2: 0.0
        }
    }
}

impl Filter for ResonantFilter {
    fn process(&mut self, x: f32) -> f32 {
        let y = x - self.b1*self.y1 - self.b2*self.y2;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}