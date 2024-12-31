const VOWELS: &str = "aeilmnouyAEINOUVY2369&{@";
const CONSONANTS: &str = "bdfghkprstvxzDSTZ4ʤʧ";

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
    pub fn build(sampa: &str) -> Result<Syllable, &'static str> {
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
                    return Err("Vowel after terminal consonant");
                }
                vowels.push(c);
                stage = 1;
            }
            else if c == '-' {
                if vowels.len() == 0 || stage == 2 {
                    return Err("- must follow a vowel");
                }
                if has_explicit_main {
                    return Err("Only one sound can be marked as the main vowel")
                }
                explicit_main = vowels.len()-1;
                has_explicit_main = true;
            }
            else {
                return Err("Illegal character '{c}");
            }
        }

        // Identify the main vowel.

        if vowels.len() == 0 {
            return Err("No vowel in syllable");
        }
        let mut main = if vowels.len() == 1 {0} else {1};
        if has_explicit_main {
            main = explicit_main;
        }
        else {
            for (i, c) in vowels.iter().enumerate() {
                if *c == 'A' || vowels[main] == 'm' || vowels[main] == 'n' || vowels[main] == 'N' {
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