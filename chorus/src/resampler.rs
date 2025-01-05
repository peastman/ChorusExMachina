// Copyright 2025 by Peter Eastman
//
// This file is part of Chorus Ex Machina.
//
// Chorus Ex Machina is free software: you can redistribute it and/or modify it under the terms
// of the GNU Lesser General Public License as published by the Free Software Foundation, either
// version 2.1 of the License, or (at your option) any later version.
//
// Chorus Ex Machina is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY;
// without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See
// the GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License along with Chorus Ex Machina.
// If not, see <https://www.gnu.org/licenses/>.

use crate::SAMPLE_RATE;

/// Convert output from the synthesizer's native sample rate (48 kHz) to a different sample rate.
/// The method used by this class is very fast and doesn't introduce latency, but the results may
/// not always be the best.  When possible, it is preferable to output at the native sample rate.
#[derive(Copy, Clone)]
pub struct Resampler {
    output_interval: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    next_output_time: f32
}

impl Resampler {
    /// Create a Resampler that converts to a specified sample rate, measured in Hz.
    pub fn new(sample_rate: f32) -> Self {
        Self {
            output_interval: SAMPLE_RATE as f32/sample_rate,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            next_output_time: 0.0
        }
    }

    /// Get whether there is output ready.
    pub fn has_output(&self) -> bool {
        self.next_output_time <= self.x2
    }

    /// Get the next output sample.  This will panic if no output is ready.
    pub fn get_output(&mut self) -> f32 {
        assert!(self.has_output(), "No output ready");
        let f = self.x2-self.next_output_time;
        let result = f*self.y1 + (1.0-f)*self.y2;
        self.next_output_time += self.output_interval;
        while self.x2 > 1.0 && self.next_output_time > 1.0 {
            self.x2 -= 1.0;
            self.next_output_time -= 1.0;
        }
        result
    }

    /// Add an input value.  This will panic if there is output waiting to be retreived.
    pub fn add_input(&mut self, y: f32) {
        assert!(!self.has_output(), "Cannot add input when output is ready");
        self.y1 = self.y2;
        self.y2 = y;
        self.x2 += 1.0;
    }
}