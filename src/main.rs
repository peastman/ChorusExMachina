pub mod synth;
mod random;

use synth::Voice;

use rodio::{OutputStream, Source};
use std::thread::sleep;
use std::time::Duration;

struct Player {
    voice: Vec<Voice>,
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        let mut sum = 0.0;
        for i in 0..self.voice.len() {
            sum += self.voice[i].generate();
        }
        Some(0.01*sum)
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
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples().fade_in(Duration::from_millis(100)));
    sleep(Duration::from_millis(15000));
}
