use chorus::voice;
use chorus::phonemes::Phonemes;
use chorus::director::{Director, Message};

use rodio::{OutputStream, Source};
use std::sync::mpsc;
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
    let (sender, receiver) = mpsc::channel();
    let mut player = Player { director: Director::new(vec![voice::Voice::new(48000)], receiver) };
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples());
    sender.send(Message::NoteOn {syllable: "Ai".to_string(), note_index: 55, velocity: 1.0});
    sleep(Duration::from_millis(2000));
    sender.send(Message::NoteOn {syllable: "o".to_string(), note_index: 60, velocity: 1.0});
    sleep(Duration::from_millis(2000));
    sender.send(Message::NoteOff);
    sleep(Duration::from_millis(10000));
}
