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

use crate::filter::{Filter, HighpassFilter};

/// This implements a harmonic exciter.  It processes an input signal through the following steps.
///
/// - A highpass filter to extract the high frequency part of the signal
/// - A nonlinearity to create more harmonics
/// - A second highpass filter
/// - Adding the result to the input signal
pub struct Exciter {
    filter1: HighpassFilter,
    filter2: HighpassFilter
}

impl Exciter {
    pub fn new(cutoff: f32) -> Self {
        Self {
            filter1: HighpassFilter::new(cutoff),
            filter2: HighpassFilter::new(2.0*cutoff)
        }
    }
    pub fn process(&mut self, x: f32, strength: f32) -> f32 {
        let boost = strength/(1.0-strength);
        let y = self.filter1.process(x);
        let y = y*(1.0+boost)/(1.0+boost*y.abs());
        let y = self.filter2.process(y);
        x + 0.5*y
    }
}