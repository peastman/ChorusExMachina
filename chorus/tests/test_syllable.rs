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

use chorus::syllable::Syllable;

#[test]
fn should_fail() {
    let bad_specs = vec!["", "k", "a~", "$i", "dk", "uko", "dado", "-a", "ak-", "a-i-"];
    for spec in bad_specs {
        if let Ok(_s) = Syllable::build(spec) {
            panic!["{spec} should have failed to parse"];
        }
    }
}

#[test]
fn should_pass() {
    let specs = vec!["a", "i", "ked", "Adk", "kdIkd", "Ai", "aim", "dui", "iu", "uio", "u-io", "uio-", "dZAtS"];
    let expected_initial_consonants = vec!["", "", "k", "", "kd", "", "", "d", "", "", "", "", "ʤ"];
    let expected_initial_vowels = vec!["", "", "", "", "", "", "a", "u", "i", "u", "", "ui", ""];
    let expected_main_vowel = vec!['a', 'i', 'e', 'A', 'I', 'A', 'i', 'i', 'u', 'i', 'u', 'o', 'A'];
    let expected_final_vowels = vec!["", "", "", "", "", "i", "m", "", "", "o", "io", "", ""];
    let expected_final_consonants = vec!["", "", "d", "dk", "kd", "", "", "", "", "", "", "", "ʧ"];
    for i in 0..specs.len() {
        let syllable = Syllable::build(specs[i]).unwrap();
        assert_eq!(expected_initial_consonants[i].chars().collect::<Vec<char>>(), syllable.initial_consonants);
        assert_eq!(expected_final_consonants[i].chars().collect::<Vec<char>>(), syllable.final_consonants);
        assert_eq!(expected_initial_vowels[i].chars().collect::<Vec<char>>(), syllable.initial_vowels);
        assert_eq!(expected_final_vowels[i].chars().collect::<Vec<char>>(), syllable.final_vowels);
        assert_eq!(expected_main_vowel[i], syllable.main_vowel);
    }
}
