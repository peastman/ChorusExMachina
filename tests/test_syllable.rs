use chorus::syllable::Syllable;

#[test]
fn should_fail() {
    let bad_specs = vec!["", "k", "a~", "$i", "lk", "uko", "lalo", "-a", "ak-", "a-i-"];
    for spec in bad_specs {
        if let Ok(_s) = Syllable::build(spec) {
            panic!["{spec} should have failed to parse"];
        }
    }
}

#[test]
fn should_pass() {
    let specs = vec!["a", "i", "kel", "Alk", "klIkd", "Ai", "aim", "dui", "iu", "uio", "u-io", "uio-"];
    let expected_initial_consonants = vec!["", "", "k", "", "kl", "", "", "d", "", "", "", ""];
    let expected_initial_vowels = vec!["", "", "", "", "", "", "a", "u", "i", "u", "", "ui"];
    let expected_main_vowel = vec!["a", "i", "e", "A", "I", "A", "i", "i", "u", "i", "u", "o"];
    let expected_final_vowels = vec!["", "", "", "", "", "i", "m", "", "", "o", "io", ""];
    let expected_final_consonants = vec!["", "", "l", "lk", "kd", "", "", "", "", "", "", ""];
    for i in 0..specs.len() {
        let syllable = Syllable::build(specs[i]).unwrap();
        assert_eq!(expected_initial_consonants[i].chars().collect::<Vec<char>>(), syllable.initial_consonants);
        assert_eq!(expected_final_consonants[i].chars().collect::<Vec<char>>(), syllable.final_consonants);
        assert_eq!(expected_initial_vowels[i].chars().collect::<Vec<char>>(), syllable.initial_vowels);
        assert_eq!(expected_final_vowels[i].chars().collect::<Vec<char>>(), syllable.final_vowels);
    }
}
