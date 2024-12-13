use chorus::voice;
use chorus::phonemes::Phonemes;
use chorus::director::{Director, Message};

use rodio::{OutputStream, Source};
use midir::MidiInput;
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

struct MidiController {
    sender: mpsc::Sender<Message>,
    last_note: u8
}

fn process_midi_message(timestamp: u64, message: &[u8], controller: &mut MidiController) {
    if message[0] == 144 {
        controller.sender.send(Message::NoteOn {syllable: "U".to_string(), note_index: message[1] as i32, velocity: message[2] as f32 / 127.0});
        controller.last_note = message[1];
    }
    else if message[0] == 128 && controller.last_note == message[1] {
        controller.sender.send(Message::NoteOff);
    }
    else if message[0] == 176 && message[1] == 1 {
        controller.sender.send(Message::SetVolume {volume: message[2] as f32 / 127.0});
    }
    else if message[0] == 224 {
        let value = message[2] as f32 + message[1] as f32 / 127.0;
        controller.sender.send(Message::SetPitchBend {semitones: (value-64.0)/32.0});
    }
}

fn main() {
    let (sender, receiver) = mpsc::channel();
    let mut player = Player { director: Director::new(vec![voice::Voice::new(48000)], receiver) };
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples());

    let mut midi_in = MidiInput::new("Chorus").unwrap();
    let in_ports = midi_in.ports();
    let in_port = &in_ports[1];
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        process_midi_message,
        MidiController {sender: sender, last_note: 255},
    ).unwrap();
    sleep(Duration::from_millis(100000));
}
