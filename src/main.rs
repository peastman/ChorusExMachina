pub mod synth;
pub mod phonemes;
mod random;
mod filter;

use synth::Voice;
use phonemes::Phonemes;

use rodio::{OutputStream, Source};
use std::thread::sleep;
use std::time::Duration;

struct Player {
    voice: Vec<Voice>
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let mut sum = 0.0;
        for i in 0..self.voice.len() {
            sum += self.voice[i].generate();
        }
        Some(0.02*sum)
    }
}

impl Source for Player {
    fn channels(&self) -> u16 {
        return 1;
    }

    fn sample_rate(&self) -> u32 {
        return 48000;
    }

    fn total_duration(&self) -> Option<Duration> {
        return None;
    }

    fn current_frame_len(&self) -> Option<usize> {
        return None;
    }
}

fn main() {
    let mut player = Player { voice: vec![synth::Voice::new(48000)] };
    let phonemes = Phonemes::new();
    let vowel = "m";
    player.voice[0].set_vocal_shape(phonemes.get_vowel_shape(vowel).unwrap(), phonemes.get_nasal_coupling(vowel));
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples().fade_in(Duration::from_millis(100)));
    sleep(Duration::from_millis(10000));
}
