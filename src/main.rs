pub mod synth;
pub mod phonemes;
mod director;
mod random;
mod filter;

use synth::Voice;
use phonemes::Phonemes;
use director::{Director, TransitionData};

use rodio::{OutputStream, Source};
use std::thread::sleep;
use std::time::Duration;

struct Player {
    director: Director
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        Some(self.director.generate())
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
    let mut director = Director::new(vec![synth::Voice::new(48000)]);
    director.add_transition(0, 10000, TransitionData::VolumeChange {start_volume: 0.0, end_volume: 1.0});
    let phonemes = Phonemes::new();
    let start_shape = phonemes.get_vowel_shape("i").unwrap();
    let end_shape = phonemes.get_vowel_shape("A").unwrap();
    let start_nasal_coupling = phonemes.get_nasal_coupling("i");
    let end_nasal_coupling = phonemes.get_nasal_coupling("A");
    director.add_transition(0, 0, TransitionData::ShapeChange {start_shape: start_shape.clone(), end_shape: start_shape.clone(), start_nasal_coupling: start_nasal_coupling, end_nasal_coupling: start_nasal_coupling});
    director.add_transition(7000, 5000, TransitionData::ShapeChange {start_shape: start_shape.clone(), end_shape: end_shape.clone(), start_nasal_coupling: start_nasal_coupling, end_nasal_coupling: end_nasal_coupling});
    let mut player = Player { director: director };
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples().fade_in(Duration::from_millis(100)));
    sleep(Duration::from_millis(10000));
}
