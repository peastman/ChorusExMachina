use chorus::phonemes::Phonemes;
use chorus::director::{Director, Message};
use chorus::VoicePart;

use rodio::{OutputStream, Source};
use midir::MidiInput;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use eframe::egui::{self, CentralPanel};
use eframe::{App, NativeOptions};

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
    pub sender: mpsc::Sender<Message>,
    pub syllable: String,
    last_note: u8,
}

fn process_midi_message(timestamp: u64, message: &[u8], data: &mut Arc<Mutex<MidiController>>) {
    let mut controller = data.lock().unwrap();
    if message[0] == 144 {
        let _ = controller.sender.send(Message::NoteOn {syllable: controller.syllable.clone(), note_index: message[1] as i32, velocity: message[2] as f32 / 127.0});
        controller.last_note = message[1];
    }
    else if message[0] == 128 && controller.last_note == message[1] {
        let _ = controller.sender.send(Message::NoteOff);
    }
    else if message[0] == 176 && message[1] == 1 {
        let _ = controller.sender.send(Message::SetVolume {volume: message[2] as f32 / 127.0});
    }
    else if message[0] == 224 {
        let value = message[2] as f32 + message[1] as f32 / 128.0;
        let _ = controller.sender.send(Message::SetPitchBend {semitones: (value-64.0)/32.0});
    }
}

struct MainGui {
    controller_ref: Arc<Mutex<MidiController>>,
    rd: f32,
    noise: f32
}

impl App for MainGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut controller = self.controller_ref.lock().unwrap();
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Syllable");
                ui.text_edit_singleline(&mut controller.syllable);
            });
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let (sender, receiver) = mpsc::channel();
    let mut player = Player { director: Director::new(VoicePart::Soprano, 1, receiver) };
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples());

    let midi_in = MidiInput::new("Chorus").unwrap();
    let in_ports = midi_in.ports();
    let in_port = &in_ports[1];
    let controller = Arc::new(Mutex::new(MidiController {sender: sender.clone(), last_note: 255, syllable: "A".to_string()}));
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        process_midi_message,
        Arc::clone(&controller),
    ).unwrap();
    let options = NativeOptions::default();
    let gui = MainGui {
        controller_ref: Arc::clone(&controller),
        rd: 2.0,
        noise: 0.01
    };
    eframe::run_native(
        "Chorus",
        options,
        Box::new(|_cc| Ok(Box::new(gui))),
    )
}
