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

mod editor;

use chorus;
use chorus::director::{Director, Message};
use chorus::resampler::Resampler;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::{Arc, Mutex, mpsc};

pub struct ChorusExMachina {
    params: Arc<ChorusExMachinaParams>,
    director: Arc<Mutex<Director>>,
    sender: Arc<Mutex<mpsc::Sender<Message>>>,
    editor_state: Arc<Mutex<editor::UIState>>,
    need_resample: bool,
    resample_left: Resampler,
    resample_right: Resampler,
    last_note: u8,
    last_dynamics: f32,
    last_vibrato: f32,
    last_intensity: f32,
    last_brightness: f32,
    last_consonant_volume: f32,
    last_attack_rate: f32,
    last_release_rate: f32,
    last_stereo_width: f32,
    last_exciter_strength: f32,
    last_time_spread: i32,
    last_vowel_delay: i32,
    last_accent: bool,
    last_phrase: i32,
    last_syllable_index: i32
}

#[derive(Params)]
struct ChorusExMachinaParams {
    #[persist = "editor_state"]
    editor_state: Arc<EguiState>,
    #[persist = "phrases"]
    pub phrases: Mutex<Vec<String>>,
    #[id = "voice_part"]
    pub voice_part: EnumParam<VoicePart>,
    #[id = "voice_count"]
    pub voice_count: IntParam,
    #[id = "dynamics"]
    pub dynamics: FloatParam,
    #[id = "vibrato"]
    pub vibrato: FloatParam,
    #[id = "intensity"]
    pub intensity: FloatParam,
    #[id = "brightness"]
    pub brightness: FloatParam,
    #[id = "consonant_volume"]
    pub consonant_volume: FloatParam,
    #[id = "attack_rate"]
    pub attack_rate: FloatParam,
    #[id = "release_rate"]
    pub release_rate: FloatParam,
    #[id = "stereo_width"]
    pub stereo_width: FloatParam,
    #[id = "exciter_strength"]
    pub exciter_strength: FloatParam,
    #[id = "time_spread"]
    pub time_spread: IntParam,
    #[id = "vowel_delay"]
    pub vowel_delay: IntParam,
    #[id = "accent"]
    pub accent: BoolParam,
    #[id = "selected_phrase"]
    pub selected_phrase: IntParam,
    #[id = "advance_syllable"]
    pub advance_syllable: BoolParam
}

#[derive(Copy, Clone, Enum, Debug, PartialEq)]
pub enum VoicePart {
    #[id = "octaves"]
    Soprano,
    #[id = "alto"]
    Alto,
    #[id = "tenor"]
    Tenor,
    #[id = "bass"]
    Bass,
}

impl Default for ChorusExMachina {
    fn default() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            params: Arc::new(ChorusExMachinaParams::default()),
            director: Arc::new(Mutex::new(Director::new(chorus::VoicePart::Soprano, 1, receiver))),
            sender: Arc::new(Mutex::new(sender)),
            editor_state: Arc::new(Mutex::new(editor::UIState::new())),
            need_resample: false,
            resample_left: Resampler::new(chorus::SAMPLE_RATE as f32),
            resample_right: Resampler::new(chorus::SAMPLE_RATE as f32),
            last_note: 255,
            last_dynamics: -1.0,
            last_vibrato: -1.0,
            last_intensity: -1.0,
            last_brightness: -1.0,
            last_consonant_volume: -1.0,
            last_attack_rate: -1.0,
            last_release_rate: -1.0,
            last_stereo_width: -1.0,
            last_exciter_strength: -1.0,
            last_time_spread: -1,
            last_vowel_delay: -1,
            last_accent: false,
            last_phrase: -1,
            last_syllable_index: -1
        }
    }
}

impl Default for ChorusExMachinaParams {
    fn default() -> Self {
        let result = Self {
            editor_state: EguiState::from_size(600, 400),
            phrases: Mutex::new(vec!["".to_string(); 128]),
            voice_part: EnumParam::new("Voice Part", VoicePart::Soprano).non_automatable(),
            voice_count: IntParam::new("Voices", 4, IntRange::Linear {min: 1, max: 8}).non_automatable(),
            dynamics: FloatParam::new("Dynamics", 1.0, FloatRange::Linear {min: 0.0, max: 1.0}),
            vibrato: FloatParam::new("Vibrato", 0.4, FloatRange::Linear {min: 0.0, max: 1.0}),
            intensity: FloatParam::new("Intensity", 0.5, FloatRange::Linear {min: 0.0, max: 1.0}),
            brightness: FloatParam::new("Brightness", 1.0, FloatRange::Linear {min: 0.0, max: 1.0}),
            consonant_volume: FloatParam::new("Consonant Volume", 0.5, FloatRange::Linear {min: 0.0, max: 1.0}),
            attack_rate: FloatParam::new("Attack Rate", 0.8, FloatRange::Linear {min: 0.0, max: 1.0}),
            release_rate: FloatParam::new("Release Rate", 0.5, FloatRange::Linear {min: 0.0, max: 1.0}),
            stereo_width: FloatParam::new("Stereo Width", 0.5, FloatRange::Linear {min: 0.0, max: 1.0}),
            exciter_strength: FloatParam::new("Exciter Strength", 0.5, FloatRange::Linear {min: 0.0, max: 1.0}),
            time_spread: IntParam::new("Time Spread", 40, IntRange::Linear {min: 0, max: 100}),
            vowel_delay: IntParam::new("Vowel Delay", 0, IntRange::Linear {min: 0, max: 250}),
            accent: BoolParam::new("Accent", false),
            selected_phrase: IntParam::new("Selected Phrase", 0, IntRange::Linear {min: 0, max: 127}),
            advance_syllable: BoolParam::new("Advance Syllable", true),
        };
        result.phrases.lock().unwrap()[0] = "A".to_string();
        result
    }
}

impl Plugin for ChorusExMachina {
    const NAME: &'static str = "Chorus Ex Machina";
    const VENDOR: &'static str = "Peter Eastman";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "peter.eastman@gmail.com";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: None,
        main_output_channels: NonZeroU32::new(2),
        aux_input_ports: &[],
        aux_output_ports: &[],
        names: PortNames::const_default(),
    }];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;
    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
        self.need_resample = buffer_config.sample_rate != chorus::SAMPLE_RATE as f32;
        self.resample_left = Resampler::new(buffer_config.sample_rate);
        self.resample_right = Resampler::new(buffer_config.sample_rate);
        let voice_part = match self.params.voice_part.value() {
            VoicePart::Soprano => chorus::VoicePart::Soprano,
            VoicePart::Alto => chorus::VoicePart::Alto,
            VoicePart::Tenor => chorus::VoicePart::Tenor,
            VoicePart::Bass => chorus::VoicePart::Bass,
        };
        let voice_count = self.params.voice_count.value() as usize;
        let _ = self.sender.lock().unwrap().send(Message::Reinitialize {voice_part: voice_part, voice_count: voice_count});
        true
    }

    fn reset(&mut self) {
        self.last_syllable_index = -1;
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        let mut director = self.director.lock().unwrap();
        let sender = self.sender.lock().unwrap();
        let mut next_event = context.next_event();
        if self.last_dynamics != self.params.dynamics.value() {
            self.last_dynamics = self.params.dynamics.value();
            let _ = sender.send(Message::SetVolume {volume: self.last_dynamics});
        }
        if self.last_vibrato != self.params.vibrato.value() {
            self.last_vibrato = self.params.vibrato.value();
            let _ = sender.send(Message::SetVibrato {vibrato: self.last_vibrato});
        }
        if self.last_intensity != self.params.intensity.value() {
            self.last_intensity = self.params.intensity.value();
            let _ = sender.send(Message::SetIntensity {intensity: self.last_intensity});
        }
        if self.last_brightness != self.params.brightness.value() {
            self.last_brightness = self.params.brightness.value();
            let _ = sender.send(Message::SetBrightness {brightness: self.last_brightness});
        }
        if self.last_consonant_volume != self.params.consonant_volume.value() {
            self.last_consonant_volume = self.params.consonant_volume.value();
            let _ = sender.send(Message::SetConsonantVolume {volume: self.last_consonant_volume});
        }
        if self.last_attack_rate != self.params.attack_rate.value() {
            self.last_attack_rate = self.params.attack_rate.value();
            let _ = sender.send(Message::SetAttackRate {attack: self.last_attack_rate});
        }
        if self.last_release_rate != self.params.release_rate.value() {
            self.last_release_rate = self.params.release_rate.value();
            let _ = sender.send(Message::SetReleaseRate {release: self.last_release_rate});
        }
        if self.last_stereo_width != self.params.stereo_width.value() {
            self.last_stereo_width = self.params.stereo_width.value();
            let _ = sender.send(Message::SetStereoWidth {width: self.last_stereo_width});
        }
        if self.last_exciter_strength != self.params.exciter_strength.value() {
            self.last_exciter_strength = self.params.exciter_strength.value();
            let _ = sender.send(Message::SetExciterStrength {strength: self.last_exciter_strength*0.9});
        }
        if self.last_time_spread != self.params.time_spread.value() {
            self.last_time_spread = self.params.time_spread.value();
            let _ = sender.send(Message::SetMaxVoiceDelay {max_delay: (self.last_time_spread*chorus::SAMPLE_RATE/1000) as i64});
        }
        if self.last_vowel_delay != self.params.vowel_delay.value() {
            self.last_vowel_delay = self.params.vowel_delay.value();
            let _ = sender.send(Message::SetMinVowelStartTime {samples: (self.last_vowel_delay*chorus::SAMPLE_RATE/1000) as i64});
        }
        if self.last_accent != self.params.accent.value() {
            self.last_accent = self.params.accent.value();
            let _ = sender.send(Message::SetAccent {accent: self.last_accent});
        }
        if self.last_phrase != self.params.selected_phrase.value() {
            self.last_phrase = self.params.selected_phrase.value();
            self.last_syllable_index = -1;
        }
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            let mut send_note_off = false;
            while let Some(event) = next_event {
                if event.timing() != sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        let phrase = self.params.phrases.lock().unwrap()[self.params.selected_phrase.value() as usize].clone();
                        let replaced = phrase.replace(".", " ");
                        let syllables: Vec<&str> = replaced.split_whitespace().collect();
                        if syllables.len() > 0 {
                            if self.params.advance_syllable.value() {
                                self.last_syllable_index = (self.last_syllable_index+1)%syllables.len() as i32;
                            }
                            let syllable_index: usize;
                            if self.last_syllable_index < 0 || self.last_syllable_index >= syllables.len() as i32 {
                                syllable_index = 0;
                            }
                            else {
                                syllable_index = self.last_syllable_index as usize;
                            }
                            let _ = sender.send(Message::NoteOn {
                                syllable: syllables[syllable_index].to_string(),
                                note_index: note as i32,
                                velocity: velocity,
                                continue_syllable: !self.params.advance_syllable.value()});
                            self.last_note = note;

                            // If we get both a NoteOn and a NoteOff and the same time, skip the NoteOff
                            // to allow legato playing.

                            send_note_off = false;
                        }
                    },
                    NoteEvent::NoteOff { note, .. } => {
                        if note == self.last_note {
                            send_note_off = true;
                        }
                    },
                    NoteEvent::MidiPitchBend { value, .. } => {
                        let _ = sender.send(Message::SetPitchBend {semitones: 4.0*(value-0.5)});
                    },
                    _ => (),
                }
                next_event = context.next_event();
            }
            if send_note_off {
                let _ = sender.send(Message::NoteOff);
            }
            let left;
            let right;
            if self.need_resample {
                while !self.resample_left.has_output() {
                    let (left2, right2) = director.generate();
                    self.resample_left.add_input(left2);
                    self.resample_right.add_input(right2);
                }
                left = self.resample_left.get_output();
                right = self.resample_right.get_output();
            }
            else {
                (left, right) = director.generate();
            }
            let mut i = 0;
            for sample in channel_samples {
                if i == 0 {
                    *sample = left;
                }
                else if i == 1 {
                    *sample = right;
                }
                i += 1;
            }
        }
        ProcessStatus::KeepAlive
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = Arc::clone(&self.params);
        let sender = Arc::clone(&self.sender);
        let state = Arc::clone(&self.editor_state);
        editor::draw_editor(params, sender, state)
    }
}

impl ClapPlugin for ChorusExMachina {
    const CLAP_ID: &'static str = "com.github.peastman.ChorusExMachina";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A physically modelled chorus synthesizer");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Instrument, ClapFeature::Synthesizer, ClapFeature::Stereo];
}

impl Vst3Plugin for ChorusExMachina {
    const VST3_CLASS_ID: [u8; 16] = *b"ChorusExMachina.";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] = &[Vst3SubCategory::Instrument, Vst3SubCategory::Synth, Vst3SubCategory::Stereo];
}

nih_export_clap!(ChorusExMachina);
nih_export_vst3!(ChorusExMachina);
