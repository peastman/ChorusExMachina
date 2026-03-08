// Copyright 2026 by Peter Eastman
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

use chorus::reverb::Reverb;
use chorus::random::Random;
use realfft::RealFftPlanner;

#[test]
fn test_reverb() {
    // Create a random input sequence and IR.

    let mut random = Random::new();
    let mut input = vec![];
    for _ in 0..200 {
        input.push(random.get_normal());
    }
    let mut ir = vec![];
    for _ in 0..120 {
        ir.push(random.get_normal());
    }

    // Compute the convolved sequence.

    let mut expected = vec![0.0; input.len()+ir.len()+10];
    for i in 0..input.len() {
        for j in 0..ir.len() {
            expected[i+j] += input[i]*ir[j];
        }
    }

    // See if the reverb produces the correct sequence.

    let mut fft_planner = RealFftPlanner::<f32>::new();
    let mut reverb = Reverb::new(&ir, &mut fft_planner);
    for i in 0..expected.len() {
        let x = if i < input.len() {input[i]} else {0.0};
        let output = reverb.process(x);
        assert!((expected[i]-output).abs() < 1e-4);
    }
}
