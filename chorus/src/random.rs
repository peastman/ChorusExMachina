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

use getrandom::getrandom;

const UNIFORM_SCALE: f32 = 1.0/(0x100000000i64 as f32);

/// This is a quick and dirty random number generator.  It is based on the
/// "even quicker generator" in Numerical Recipes.  Its statistical properties
/// aren't great, but it's fine for our purposes, and it's very fast.
pub struct Random {
    i: u32,
    next_normal: f32,
    next_normal_valid: bool
}

impl Random {
    pub fn new() -> Self {
        // Select a seed.

        let mut data = [0u8; 4];
        let mut seed = 0;
        if let Ok(_) = getrandom(&mut data) {
            for i in 0..4 {
                seed += (data[i] as u32) << 8*i;
            }
        }
        else {
            // This should only happen in strange situations when something went wrong
            // at the OS level.  Just use 0.
        }
        Self {i: seed, next_normal: 0.0, next_normal_valid: false}
    }

    /// Get a random integer.
    pub fn get_int(&mut self) -> u32 {
        self.i = ((self.i as u64)*1664525u64 + 1013904223u64) as u32;
        self.i
    }

    /// Get a random value, uniformly distributed between 0.0 and 1.0.
    pub fn get_uniform(&mut self) -> f32 {
        UNIFORM_SCALE * (self.get_int() as f32)
    }

    /// Get a random value from a normal distribution.
    pub fn get_normal(&mut self) -> f32 {
        if self.next_normal_valid {
            self.next_normal_valid = false;
            return self.next_normal;
        }
        loop {
            let x = 2.0*self.get_uniform()-1.0;
            let y = 2.0*self.get_uniform()-1.0;
            let r2 = x*x + y*y;
            if r2 < 1.0 && r2 != 0.0 {
                let multiplier = (-2.0*r2.ln()/r2).sqrt();
                self.next_normal = y*multiplier;
                self.next_normal_valid = true;
                return x*multiplier;
            }
        }
    }

    /// Get count indices randomly selected from 0..range.
    pub fn get_indices(&mut self, count: usize, range: usize) -> Vec<usize> {
        let mut samples: Vec<usize> = (0..range).collect();
        for i in 0..count {
            let j = self.get_int() >> 16;
            samples.swap(i, j as usize % range);
        }
        samples[..count].to_vec()
    }
}
