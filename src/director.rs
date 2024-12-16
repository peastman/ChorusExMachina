use crate::voice::Voice;
use crate::phonemes::Phonemes;
use crate::syllable::Syllable;
use crate::VoicePart;
use std::sync::{Arc, mpsc};

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

struct Consonant {
    start: i64,
    end: i64,
    samples: Arc<Vec<i16>>
}

pub struct Director {
    voices: Vec<Voice>,
    voice_part: VoicePart,
    lowest_note: i32,
    highest_note: i32,
    phonemes: Phonemes,
    step: i64,
    transitions: Vec<Transition>,
    current_note: Option<Note>,
    consonants: Vec<Consonant>,
    volume: f32,
    envelope: f32,
    frequency: f32,
    bend: f32,
    off_after_step: i64,
    shape_after_transitions: Vec<f32>,
    nasal_coupling_after_transitions: f32,
    envelope_after_transitions: f32,
    frequency_after_transitions: f32,
    message_receiver: mpsc::Receiver<Message>
}

impl Director {
    pub fn new(voice_part: VoicePart, voice_count: i32, message_receiver: mpsc::Receiver<Message>) -> Self {
        let mut voices = vec![];
        for _i in 0..voice_count {
            voices.push(Voice::new(voice_part, 48000));
        }
        let vocal_length;
        let lowest_note;
        let highest_note;
        match voice_part {
            VoicePart::Soprano => {
                vocal_length = 42;
                lowest_note = 57;
                highest_note = 88;
            }
            VoicePart::Alto => {
                vocal_length = 44;
                lowest_note = 48;
                highest_note = 79;
            }
            VoicePart::Tenor => {
                vocal_length = 46;
                lowest_note = 43;
                highest_note = 72;
            }
            VoicePart::Bass => {
                vocal_length = 48;
                lowest_note = 36;
                highest_note = 67;
            }
        }
        Self {
            voices: voices,
            voice_part: voice_part.clone(),
            lowest_note: lowest_note,
            highest_note: highest_note,
            phonemes: Phonemes::new(voice_part),
            step: 0,
            transitions: vec![],
            current_note: None,
            consonants: vec![],
            volume: 1.0,
            envelope: 0.0,
            frequency: 1.0,
            bend: 1.0,
            off_after_step: 0,
            shape_after_transitions: vec![0.0; vocal_length],
            nasal_coupling_after_transitions: 0.0,
            envelope_after_transitions: 0.0,
            frequency_after_transitions: 0.0,
            message_receiver: message_receiver
        }
    }

    fn note_on(&mut self, syllable: &str, note_index: i32, velocity: f32) -> Result<(), &'static str> {
        // If the note index is outside the range of this voice part, just stop the current
        // note and exit.

        if note_index < self.lowest_note || note_index > self.highest_note {
            if self.current_note.is_some() {
                self.note_off();
            }
            return Ok(());
        }

        // Prepare for playing the note.

        let new_syllable = Syllable::build(syllable)?;
        let frequency = 440.0 * f32::powf(2.0, (note_index-69) as f32/12.0);
        let mut delay = 0;
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }

        // Identify any consonants we need to play, either final consonants from the
        // previous note or initial consonants from the new note.

        let mut consonants = Vec::new();
        if let Some(note) = &self.current_note {
            for c in &note.syllable.final_consonants {
                consonants.push(*c);
            }
        }
        for c in &new_syllable.initial_consonants {
            consonants.push(*c);
        }

        // Play any final vowels from the previous note.

        let mut current_note_index = 0;
        let mut legato = false;
        if let Some(note) = &self.current_note {
            current_note_index = note.note_index;
            for c in &note.syllable.final_vowels.clone() {
                delay = self.add_transient_vowel(delay, *c);
            }
            if consonants.len() == 0 {
                legato = true;
            }
            else {
                // Since there are consonants between the notes, we can't play them legato.
                // Turn off the previous note.

                self.add_transition(delay, 3000, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: 0.0});
            }
        }
        if legato {
            // Smoothly transition between the two notes.

            let transition_time = (1100 + 270*(current_note_index-note_index).abs()) as i64;
            self.add_transition(delay, transition_time, TransitionData::FrequencyChange {start_frequency: self.frequency_after_transitions, end_frequency: frequency});
            let min_envelope = f32::powf(0.9, (current_note_index-note_index).abs() as f32);
            self.add_transition(delay, transition_time/2, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: min_envelope});
            self.add_transition(delay+transition_time/2, transition_time/2, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: 1.0});
            delay += transition_time;
        }
        else {
            // Play any consonants, then start up the new note.

            for c in &consonants {
                delay = self.add_consonant(delay, *c);
            }
            self.add_transition(delay, 0, TransitionData::FrequencyChange {start_frequency: frequency, end_frequency: frequency});
            self.add_transition(delay, 2000, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: 1.0});
        }

        // Play any initial vowels.

        for c in &new_syllable.initial_vowels {
            delay = self.add_transient_vowel(delay, *c);
        }

        // Start the main vowel playing.

        let shape = self.phonemes.get_vowel_shape(new_syllable.main_vowel).unwrap();
        let nasal_coupling = self.phonemes.get_nasal_coupling(new_syllable.main_vowel);
        let transition_time = if self.current_note.is_some() || new_syllable.initial_vowels.len() > 0 || consonants.len() > 0 {2000} else {0};
        let start_shape = match consonants.last() {
            Some(c) => self.phonemes.get_vowel_shape(*c).unwrap().clone(),
            None => self.shape_after_transitions.clone()
        };
        self.add_transition(delay, transition_time, TransitionData::ShapeChange {
            start_shape: start_shape,
            end_shape: shape.clone(),
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: nasal_coupling
        });

        // Record the note we're now playing.

        let note = Note {
            syllable: new_syllable,
            note_index: note_index,
            frequency: frequency,
            velocity: velocity
        };
        self.current_note = Some(note);
        self.update_sound();
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
        let mut consonants = Vec::new();
        if let Some(note) = &self.current_note {
            for c in &note.syllable.final_consonants {
                consonants.push(*c);
            }
        }
        if consonants.len() > 0 {
            self.add_transition(delay, 3000, TransitionData::ShapeChange {
                start_shape: self.shape_after_transitions.clone(),
                end_shape: self.phonemes.get_vowel_shape(*consonants.last().unwrap()).unwrap().clone(),
                start_nasal_coupling: self.nasal_coupling_after_transitions,
                end_nasal_coupling: 0.0
            });
            for c in &consonants {
                delay = self.add_consonant(delay, *c);
            }
        }
        self.current_note = None;
    }

    fn add_transient_vowel(&mut self, delay: i64, c: char) -> i64 {
        let shape = self.phonemes.get_vowel_shape(c).unwrap();
        let nasal_coupling = self.phonemes.get_nasal_coupling(c);
        self.add_transition(delay, 2000, TransitionData::ShapeChange {
            start_shape: self.shape_after_transitions.clone(),
            end_shape: shape.clone(),
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: nasal_coupling
        });
        delay+2000
    }

    fn add_consonant(&mut self, delay: i64, c: char) -> i64 {
        let samples = self.phonemes.get_consonant_samples(c).unwrap();
        let duration = samples.len() as i64;
        let consonant = Consonant {start: self.step+delay, end: self.step+delay+duration, samples: samples };
        self.consonants.push(consonant);
        delay+3000
    }

    fn add_transition(&mut self, delay: i64, duration: i64, data: TransitionData) {
        let transition = Transition { start: self.step+delay, end: self.step+delay+duration, data: data };
        match &transition.data {
            TransitionData::EnvelopeChange {start_envelope: _, end_envelope} => {
                self.envelope_after_transitions = *end_envelope;
            }
            TransitionData::ShapeChange {start_shape: _, end_shape, start_nasal_coupling: _, end_nasal_coupling} => {
                self.shape_after_transitions = end_shape.clone();
                self.nasal_coupling_after_transitions = *end_nasal_coupling;
            }
            TransitionData::FrequencyChange {start_frequency: _, end_frequency} => {
                self.frequency_after_transitions = *end_frequency;
            }
        }
        self.transitions.push(transition);
    }

    pub fn generate(&mut self) -> f32 {
        if self.step%200 == 0 {
            self.process_messages();
            self.update_transitions();
            self.consonants.retain(|t| self.step < t.end);
        }
        self.step += 1;
        if self.envelope > 0.0 {
            self.off_after_step = self.step+500;
        }
        let mut sum = 0.0;
        if self.step < self.off_after_step {
            for voice in &mut self.voices {
                sum += voice.generate(self.step);
            }
        }
        let mut sample_sum = 0;
        for consonant in &self.consonants {
            if self.step >= consonant.start && self.step < consonant.end {
                let i = (self.step-consonant.start) as usize;
                sample_sum += consonant.samples[i] as i32;
            }
        }
        sum += 3.0*sample_sum as f32 / 32768.0;
        0.08*sum
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
                            self.update_sound();
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
        let actual_volume = 0.1+0.9*self.volume;
        for voice in &mut self.voices {
            voice.set_volume(actual_volume*self.envelope);
        }
    }

    fn update_frequency(&mut self) {
        for voice in &mut self.voices {
            voice.set_frequency(self.frequency*self.bend);
        }
    }

    fn update_sound(&mut self) {
        let noise = 0.03*(1.0-self.volume)*(1.0-self.volume);
        for voice in &mut self.voices {
            voice.set_noise(noise);
        }
        if let Some(note) = &self.current_note {
            let x = (self.highest_note-note.note_index) as f32 / (self.highest_note-self.lowest_note) as f32;
            let rd = 1.4 + 0.4*x + 0.2*self.volume;
            for voice in &mut self.voices {
                voice.set_rd(rd);
            }
        }
    }
}