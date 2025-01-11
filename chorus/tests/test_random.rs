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

use chorus::random::Random;

#[test]
fn test_unique_seeds() {
    // Every random generator should have a different seed and produce different values.

    let mut rand1 = Random::new();
    let mut rand2 = Random::new();
    let mut rand3 = Random::new();
    for _ in 0..10 {
        let v1 = rand1.get_int();
        let v2 = rand2.get_int();
        let v3 = rand3.get_int();
        assert!(v1 != v2);
        assert!(v1 != v3);
        assert!(v2 != v3);
    }
}

#[test]
fn test_bit_distributions() {
    // Every output bit should be set half the time.

    let mut rand = Random::new();
    let mut count = [0; 32];
    for _ in 0..10000 {
        let v = rand.get_int();
        for i in 0..32 {
            count[i] += (v>>i) & 1;
        }
    }
    for c in count {
        assert!(c > 4000 && c < 6000);
    }
}

#[test]
fn test_uniform_distribution() {
    let mut rand = Random::new();
    let mut count = [0; 10];
    for _ in 0..10000 {
        let v = rand.get_uniform();
        assert!(v >= 0.0 && v < 1.0);
        count[(v*10.0) as usize] += 1;
    }
    for c in count {
        assert!(c > 800 && c < 1200);
    }
}

#[test]
fn test_normal_distribution() {
    // Compute the first four moments of the distribution.

    let mut rand = Random::new();
    let mut moments = [0.0; 4];
    for _ in 0..10000 {
        let v = rand.get_normal();
        moments[0] += v;
        moments[1] += v*v;
        moments[2] += v*v*v;
        moments[3] += v*v*v*v;
    }
    for i in 0..4 {
        moments[i] /= 10000.0;
    }

    // Test the cumulants.

    let c2 = moments[1]-moments[0].powf(2.0);
    let c3 = moments[2]-3.0*moments[1]*moments[0]+2.0*moments[0].powf(2.0);
    let c4 = moments[3]-4.0*moments[2]*moments[0]-3.0*moments[1]*moments[1]+12.0*moments[1]*moments[0]*moments[0]-6.0*moments[0].powf(3.0);
    assert!(moments[0].abs() < 0.04);
    assert!((c2-1.0).abs() < 0.2);
    assert!(c3.abs() < 0.4);
    assert!(c4.abs() < 0.4);
}
