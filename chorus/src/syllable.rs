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

const VOWELS: &str = "aeilmnouyAEINOUVY239&{@";
const CONSONANTS: &str = "bdfghjkprstvwxzCDSTZ4ʤʧ";

/// A Syllable consists of:
///
/// - zero or more initial consonants
/// - zero or more initial vowels
/// - a single main vowel
/// - zero or more final vowels
/// - zero or final consonants
///
/// Syllables are created by calling Syllable::build(), which parses an X-SAMPA description.
pub struct Syllable {
    pub initial_consonants: Vec<char>,
    pub initial_vowels: Vec<char>,
    pub main_vowel: char,
    pub final_vowels: Vec<char>,
    pub final_consonants: Vec<char>
}

impl Syllable {
    pub fn build(sampa: &str) -> Result<Syllable, String> {
        // First split the string into initial consonants, vowels, and final consonants.

        let sampa = sampa.replace("tS", "ʧ").replace("dZ", "ʤ").replace("r", "4r");
        let mut initial_consonants: Vec<char> = Vec::new();
        let mut vowels: Vec<char> = Vec::new();
        let mut final_consonants: Vec<char> = Vec::new();
        let mut stage = 0;
        let mut explicit_main = 0;
        let mut has_explicit_main = false;
        for c in sampa.chars() {
            if CONSONANTS.contains(c) {
                if stage == 0 {
                    initial_consonants.push(c);
                }
                else {
                    final_consonants.push(c);
                    stage = 2;
                }
            }
            else if VOWELS.contains(c) {
                if stage == 2 {
                    return Err("Vowel after final consonant".to_string());
                }
                vowels.push(c);
                stage = 1;
            }
            else if c == '-' {
                if vowels.len() == 0 || stage == 2 {
                    return Err("- must follow a vowel".to_string());
                }
                if has_explicit_main {
                    return Err("Only one sound can be marked as the main vowel".to_string())
                }
                explicit_main = vowels.len()-1;
                has_explicit_main = true;
            }
            else {
                return Err(format!("Illegal character '{}'", c));
            }
        }

        // Identify the main vowel.

        if vowels.len() == 0 {
            return Err("No vowel in syllable".to_string());
        }
        let mut main = if vowels.len() == 1 {0} else {1};
        if has_explicit_main {
            main = explicit_main;
        }
        else {
            for (i, c) in vowels.iter().enumerate() {
                if *c == 'A' || *c == 'a' || vowels[main] == 'm' || vowels[main] == 'n' || vowels[main] == 'N' {
                    main = i;
                }
                else if (vowels[main] == 'l' || vowels[main] == '3') && (*c != 'm' && *c != 'n' && *c != 'N') {
                    main = i;
                }
            }
        }

        // Create the return value.

        let syllable = Syllable {
            initial_consonants: initial_consonants,
            initial_vowels: vowels[..main].iter().cloned().collect(),
            main_vowel: vowels[main],
            final_vowels: vowels[main+1..].iter().cloned().collect(),
            final_consonants: final_consonants
        };
        Ok(syllable)
    }
}