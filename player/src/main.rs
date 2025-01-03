use chorus::director::{Director, Message};
use chorus::VoicePart;
use chorus::phonemes::Phonemes;
use chorus::SAMPLE_RATE;

use rodio::{OutputStream, Source};
use midir::MidiInput;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use eframe::egui::{self, CentralPanel};
use eframe::{App, NativeOptions};

struct Player {
    director: Director,
    next_output: f32,
    has_next: bool
}

impl Iterator for Player {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        if self.has_next {
            self.has_next = false;
            return Some(self.next_output);
        }
        let (left, right) = self.director.generate();
        self.next_output = right;
        self.has_next = true;
        Some(left)
    }
}

impl Source for Player {
    fn channels(&self) -> u16 {
        return 2;
    }

    fn sample_rate(&self) -> u32 {
        return SAMPLE_RATE as u32;
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
    pub phrase: String,
    syllables: Vec<String>,
    next_syllable: usize,
    last_note: u8,
}

impl MidiController {
    fn new(sender: mpsc::Sender<Message>, phrase: &str) -> Self {
        Self {
            sender: sender,
            phrase: phrase.to_string(),
            syllables : phrase.split_whitespace().map(str::to_string).collect(),
            next_syllable: 0,
            last_note: 255
        }
    }
}

fn process_midi_message(_timestamp: u64, message: &[u8], data: &mut Arc<Mutex<MidiController>>) {
    let mut controller = data.lock().unwrap();
    if message[0] == 144 && controller.syllables.len() > 0 {
        let _ = controller.sender.send(Message::NoteOn {syllable: controller.syllables[controller.next_syllable].clone(), note_index: message[1] as i32, velocity: message[2] as f32 / 127.0});
        controller.next_syllable = (controller.next_syllable+1)%controller.syllables.len();
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
    voice_part: VoicePart,
    voice_count: usize,
    vowel_delay: i64,
    vowel_transition_time: i64,
    consonant_delay: i64,
    consonant_transition_time: i64,
    consonant_on_time: i64,
    consonant_off_time: i64,
    consonant_volume2: f32,
    consonant_position: usize,
    consonant_frequency: f32,
    consonant_bandwidth: f32,
    brightness: f32,
    consonant_volume: f32
}

impl App for MainGui {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut controller = self.controller_ref.lock().unwrap();
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Phrase");
                let response = ui.text_edit_singleline(&mut controller.phrase);
                if response.changed() {
                    controller.syllables = controller.phrase.split_whitespace().map(str::to_string).collect();
                    controller.next_syllable = 0;
                    if controller.phrase.len() > 0 {
                        let phonemes = Phonemes::new(self.voice_part);
                        let cons = controller.phrase.chars().next().unwrap();
                        if let Some(c) = phonemes.get_consonant(cons) {
                            self.consonant_delay = c.delay;
                            self.vowel_transition_time = c.transition_time;
                            self.consonant_on_time = c.on_time;
                            self.consonant_off_time = c.off_time;
                            self.consonant_volume2 = c.volume;
                            self.consonant_position = c.position;
                            let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
                        }
                    }
                }
            });
            let old_part = self.voice_part;
            let old_voices = self.voice_count;
            ui.horizontal(|ui| {
                egui::ComboBox::from_label("Voice Part")
                    .selected_text(format!("{:?}", self.voice_part))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.voice_part, VoicePart::Soprano, "Soprano");
                        ui.selectable_value(&mut self.voice_part, VoicePart::Alto, "Alto");
                        ui.selectable_value(&mut self.voice_part, VoicePart::Tenor, "Tenor");
                        ui.selectable_value(&mut self.voice_part, VoicePart::Bass, "Bass");
                    });
                ui.add_space(10.0);
                ui.add(egui::Slider::new(&mut self.voice_count, 1..=8).text("Voices"))
            });
            if old_part != self.voice_part || old_voices != self.voice_count {
                let _ = controller.sender.send(Message::Reinitialize {voice_part: self.voice_part, voice_count: self.voice_count});
            }
            if ui.add(egui::Slider::new(&mut self.vowel_delay, 0..=10000).text("Vowel Delay")).dragged() {
                let _ = controller.sender.send(Message::SetDelays {vowel_delay: self.vowel_delay, vowel_transition_time: self.vowel_transition_time, consonant_delay: self.consonant_delay, consonant_transition_time: self.consonant_transition_time});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_delay, 0..=10000).text("Consonant Delay")).dragged() {
                let _ = controller.sender.send(Message::SetDelays {vowel_delay: self.vowel_delay, vowel_transition_time: self.vowel_transition_time, consonant_delay: self.consonant_delay, consonant_transition_time: self.consonant_transition_time});
            }
            if ui.add(egui::Slider::new(&mut self.vowel_transition_time, 0..=10000).text("Vowel Transition Time")).dragged() {
                let _ = controller.sender.send(Message::SetDelays {vowel_delay: self.vowel_delay, vowel_transition_time: self.vowel_transition_time, consonant_delay: self.consonant_delay, consonant_transition_time: self.consonant_transition_time});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_on_time, 0..=4000).text("Consonant On Time")).dragged() {
                let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_off_time, 0..=4000).text("Consonant Off Time")).dragged() {
                let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_volume2, 0.0..=0.1).text("Consonant Volume")).dragged() {
                let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_position, 0..=46).text("Consonant Position")).dragged() {
                let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_frequency, 100.0..=5000.0).text("Consonant Frequency")).dragged() {
                let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_bandwidth, 100.0..=6000.0).text("Consonant Bandwidth")).dragged() {
                let _ = controller.sender.send(Message::SetConsonants {on_time: self.consonant_on_time, off_time: self.consonant_off_time, volume: self.consonant_volume2, position: self.consonant_position, frequency: self.consonant_frequency, bandwidth: self.consonant_bandwidth});
            }
            if ui.add(egui::Slider::new(&mut self.brightness, 0.0..=1.0).text("Brightness")).dragged() {
                let _ = controller.sender.send(Message::SetBrightness {brightness: self.brightness});
            }
            if ui.add(egui::Slider::new(&mut self.consonant_volume, 0.0..=1.0).text("Consonant Volume")).dragged() {
                let _ = controller.sender.send(Message::SetConsonantVolume {volume: self.consonant_volume});
            }
        });
    }
}

fn main() -> Result<(), eframe::Error> {
    let (sender, receiver) = mpsc::channel();
    let player = Player { director: Director::new(VoicePart::Alto, 4, receiver), next_output: 0.0, has_next: false };
    let (_stream, handle) = OutputStream::try_default().unwrap();
    let _result = handle.play_raw(player.convert_samples());

    let midi_in = MidiInput::new("Chorus").unwrap();
    let in_ports = midi_in.ports();
    let in_port = &in_ports[1];
    let controller = Arc::new(Mutex::new(MidiController::new(sender.clone(), "A")));
    let _conn_in = midi_in.connect(
        in_port,
        "midir-read-input",
        process_midi_message,
        Arc::clone(&controller),
    ).unwrap();
    let options = NativeOptions::default();
    let gui = MainGui {
        controller_ref: Arc::clone(&controller),
        voice_part: VoicePart::Alto,
        voice_count: 4,
        vowel_delay: 0,
        vowel_transition_time: 2000,
        consonant_delay: 3000,
        consonant_transition_time: 1000,
        consonant_on_time: 1000,
        consonant_off_time: 1000,
        consonant_volume2: 0.1,
        consonant_position: 40,
        consonant_frequency: 2000.0,
        consonant_bandwidth: 3000.0,
        brightness: 1.0,
        consonant_volume: 0.5
    };
    eframe::run_native(
        "Chorus",
        options,
        Box::new(|_cc| Ok(Box::new(gui))),
    )
}
