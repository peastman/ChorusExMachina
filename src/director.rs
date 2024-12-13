use crate::voice::Voice;
use crate::phonemes::Phonemes;
use crate::syllable::Syllable;
use std::sync::mpsc;

pub enum Message {
    NoteOn {syllable: String, note_index: i32, velocity: f32},
    NoteOff,
    SetVolume {volume: f32},
    SetPitchBend {semitones: f32}
}

struct Transition {
    start: i64,
    end: i64,
    data: TransitionData
}

enum TransitionData {
    EnvelopeChange {start_envelope: f32, end_envelope: f32},
    ShapeChange {start_shape: Vec<f32>, end_shape: Vec<f32>, start_nasal_coupling: f32, end_nasal_coupling: f32},
    FrequencyChange {start_frequency: f32, end_frequency: f32}
}

struct Note {
    syllable: Syllable,
    note_index: i32,
    frequency: f32,
    velocity: f32
}

pub struct Director {
    voices: Vec<Voice>,
    phonemes: Phonemes,
    step: i64,
    transitions: Vec<Transition>,
    current_note: Option<Note>,
    volume: f32,
    envelope: f32,
    frequency: f32,
    bend: f32,
    shape_after_transitions: Vec<f32>,
    nasal_coupling_after_transitions: f32,
    envelope_after_transitions: f32,
    frequency_after_transitions: f32,
    message_receiver: mpsc::Receiver<Message>
}

impl Director {
    pub fn new(voices: Vec<Voice>, message_receiver: mpsc::Receiver<Message>) -> Self {
        Self {
            voices: voices,
            phonemes: Phonemes::new(),
            step: 0,
            transitions: vec![],
            current_note: None,
            volume: 1.0,
            envelope: 0.0,
            frequency: 1.0,
            bend: 1.0,
            shape_after_transitions: vec![0.0; 44],
            nasal_coupling_after_transitions: 0.0,
            envelope_after_transitions: 0.0,
            frequency_after_transitions: 0.0,
            message_receiver: message_receiver
        }
    }

    fn note_on(&mut self, syllable: &str, note_index: i32, velocity: f32) -> Result<(), &'static str> {
        let new_syllable = Syllable::build(syllable)?;
        let frequency = 440.0 * f32::powf(2.0, (note_index-69) as f32/12.0);
        let mut delay = 0;
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }
        if let Some(note) = &self.current_note {
            for c in &note.syllable.final_vowels.clone() {
                delay = self.add_transient_vowel(delay, *c);
            }
            self.add_transition(delay, 2000, TransitionData::FrequencyChange {start_frequency: self.frequency_after_transitions, end_frequency: frequency});
        }
        else {
            self.add_transition(delay, 0, TransitionData::FrequencyChange {start_frequency: frequency, end_frequency: frequency});
            self.add_transition(delay, 2000, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: 1.0});
        }
        for c in &new_syllable.initial_vowels {
            delay = self.add_transient_vowel(delay, *c);
        }
        let shape = self.phonemes.get_vowel_shape(new_syllable.main_vowel).unwrap();
        let nasal_coupling = self.phonemes.get_nasal_coupling(new_syllable.main_vowel);
        let transition_time = if self.current_note.is_some() || new_syllable.initial_vowels.len() > 0 {7000} else {0};
        self.add_transition(delay, transition_time, TransitionData::ShapeChange {
            start_shape: self.shape_after_transitions.clone(),
            end_shape: shape.clone(),
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: nasal_coupling
        });
        let note = Note {
            syllable: new_syllable,
            note_index: note_index,
            frequency: frequency,
            velocity: velocity
        };
        self.current_note = Some(note);
        Ok(())
    }

    fn note_off(&mut self) {
        let mut delay = 0;
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }
        if let Some(note) = &self.current_note {
            for c in &note.syllable.final_vowels.clone() {
                delay = self.add_transient_vowel(delay, *c);
            }
        }
        self.add_transition(delay, 3000, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: 0.0});
        self.current_note = None;
    }

    fn add_transient_vowel(&mut self, delay: i64, c: char) -> i64 {
        let shape = self.phonemes.get_vowel_shape(c).unwrap();
        let nasal_coupling = self.phonemes.get_nasal_coupling(c);
        self.add_transition(delay, 5000, TransitionData::ShapeChange {
            start_shape: self.shape_after_transitions.clone(),
            end_shape: shape.clone(),
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: nasal_coupling
        });
        delay+8000
    }

    fn add_transition(&mut self, delay: i64, duration: i64, data: TransitionData) {
        let transition = Transition { start: self.step+delay, end: self.step+delay+duration, data: data };
        match &transition.data {
            TransitionData::EnvelopeChange {start_envelope, end_envelope} => {
                self.envelope_after_transitions = *end_envelope;
            }
            TransitionData::ShapeChange {start_shape, end_shape, start_nasal_coupling, end_nasal_coupling} => {
                self.shape_after_transitions = end_shape.clone();
                self.nasal_coupling_after_transitions = *end_nasal_coupling;
            }
            TransitionData::FrequencyChange {start_frequency, end_frequency} => {
                self.frequency_after_transitions = *end_frequency;
            }
        }
        self.transitions.push(transition);
    }

    pub fn generate(&mut self) -> f32 {
        if self.step%200 == 0 {
            self.process_messages();
            self.update_transitions();
        }
        let mut sum = 0.0;
        for voice in &mut self.voices {
            sum += voice.generate(self.step);
        }
        self.step += 1;
        0.04*sum
    }

    fn process_messages(&mut self) {
        loop {
            match self.message_receiver.try_recv() {
                Ok(message) => {
                    match message {
                        Message::NoteOn {syllable, note_index, velocity} => {
                            self.note_on(&syllable, note_index, velocity);
                        }
                        Message::NoteOff => {
                            self.note_off();
                        }
                        Message::SetVolume {volume} => {
                            self.volume = volume;
                            self.update_volume();
                        }
                        Message::SetPitchBend {semitones} => {
                            self.bend = f32::powf(2.0, semitones as f32/12.0);
                            self.update_frequency();
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    fn update_transitions(&mut self) {
        for i in 0..self.transitions.len() {
            let transition = &self.transitions[i];
            if self.step >= transition.start {
                let fraction = (self.step-transition.start) as f32 / (transition.end-transition.start) as f32;
                let weight2 = if self.step < transition.end {0.5-0.5*(fraction*std::f32::consts::PI).cos()} else {1.0};
                let weight1 = 1.0-weight2;
                match &transition.data {
                    TransitionData::EnvelopeChange {start_envelope, end_envelope} => {
                        self.envelope = weight1*start_envelope + weight2*end_envelope;
                        self.update_volume();
                    }
                    TransitionData::ShapeChange {start_shape, end_shape, start_nasal_coupling, end_nasal_coupling} => {
                        for voice in &mut self.voices {
                            let n = start_shape.len();
                            let mut shape = vec![0.0; n];
                            for i in 0..n {
                                shape[i] = weight1*start_shape[i] + weight2*end_shape[i];
                            }
                            voice.set_vocal_shape(&shape, weight1*start_nasal_coupling + weight2*end_nasal_coupling);
                        }
                    }
                    TransitionData::FrequencyChange {start_frequency, end_frequency} => {
                        self.frequency = weight1*start_frequency + weight2*end_frequency;
                        self.update_frequency();
                    }
                }
            }
        }
        self.transitions.retain(|t| self.step < t.end);
    }

    fn update_volume(&mut self) {
        for voice in &mut self.voices {
            voice.set_volume(self.volume*self.envelope);
        }
    }

    fn update_frequency(&mut self) {
        for voice in &mut self.voices {
            voice.set_frequency(self.frequency*self.bend);
        }
    }
}