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

use chorus::resampler::Resampler;
use chorus::SAMPLE_RATE;
use std::f32::consts::PI;

fn test_for_output_rate(output_rate: usize) {
    let input_rate = SAMPLE_RATE;
    let mut resampler = Resampler::new(output_rate as f32);
    let mut output = Vec::new();

    // Generate one cycle of a sine wave at the input rate.

    for i in 0..input_rate {
        while resampler.has_output() {
            output.push(resampler.get_output());
        }
        resampler.add_input(2.0*PI*(i as f32)/input_rate as f32);
    }

    // The number of output samples should equal the output rate, to within +/- 1.

    assert!((output.len() as i32 - output_rate as i32).abs() < 2);

    // The output should be a sine wave at the output rate.

    for i in 0..output.len() {
        let expected = 2.0*PI*(i as f32)/output_rate as f32;
        assert!((output[i]-expected).abs() < 0.001);
    }
}

#[test]
fn test_downsample() {
    test_for_output_rate(44100);
}

#[test]
fn test_upsample() {
    test_for_output_rate(96000);
}
