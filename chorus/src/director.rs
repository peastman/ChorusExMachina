use crate::voice::Voice;
use crate::filter::{Filter, ResonantFilter};
use crate::phonemes::{Consonant, Phonemes};
use crate::random::Random;
use crate::syllable::Syllable;
use crate::VoicePart;
use std::f32::consts::PI;
use std::sync::mpsc;

pub enum Message {
    NoteOn {syllable: String, note_index: i32, velocity: f32},
    NoteOff,
    SetVolume {volume: f32},
    SetPitchBend {semitones: f32},
    SetStereoWidth {width: f32},
    SetDelays {vowel_delay: i64, vowel_transition_time: i64, consonant_delay: i64, consonant_transition_time: i64},
    SetConsonants {on_time: i64, off_time: i64, volume: f32, position: usize, frequency: f32, bandwidth: f32}
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
    voice_part: VoicePart,
    lowest_note: i32,
    highest_note: i32,
    phonemes: Phonemes,
    random: Random,
    step: i64,
    transitions: Vec<Transition>,
    current_note: Option<Note>,
    consonants: Vec<Consonant>,
    consonant_delays: Vec<i64>,
    volume: f32,
    envelope: f32,
    frequency: f32,
    bend: f32,
    off_after_step: i64,
    shape_after_transitions: Vec<f32>,
    nasal_coupling_after_transitions: f32,
    envelope_after_transitions: f32,
    frequency_after_transitions: f32,
    message_receiver: mpsc::Receiver<Message>,
    stereo_width: f32,
    voice_pan: Vec<f32>,
    vowel_delay: i64,
    vowel_transition_time: i64,
    consonant_delay: i64,
    consonant_transition_time: i64,
    consonant_on_time: i64,
    consonant_off_time: i64,
    consonant_volume: f32,
    consonant_position: usize,
    consonant_frequency: f32,
    consonant_bandwidth: f32
}

impl Director {
    pub fn new(voice_part: VoicePart, voice_count: usize, message_receiver: mpsc::Receiver<Message>) -> Self {
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
                vocal_length = 45;
                lowest_note = 48;
                highest_note = 79;
            }
            VoicePart::Tenor => {
                vocal_length = 48;
                lowest_note = 43;
                highest_note = 72;
            }
            VoicePart::Bass => {
                vocal_length = 50;
                lowest_note = 36;
                highest_note = 67;
            }
        }
        let mut result = Self {
            voices: voices,
            voice_part: voice_part.clone(),
            lowest_note: lowest_note,
            highest_note: highest_note,
            phonemes: Phonemes::new(voice_part),
            random: Random::new(),
            step: 0,
            transitions: vec![],
            current_note: None,
            consonants: vec![],
            consonant_delays: vec![0; voice_count],
            volume: 1.0,
            envelope: 0.0,
            frequency: 1.0,
            bend: 1.0,
            off_after_step: 0,
            shape_after_transitions: vec![0.0; vocal_length],
            nasal_coupling_after_transitions: 0.0,
            envelope_after_transitions: 0.0,
            frequency_after_transitions: 0.0,
            message_receiver: message_receiver,
            stereo_width: 0.3,
            voice_pan: vec![0.0; voice_count],
            vowel_delay: 0,
            vowel_transition_time: 3300,
            consonant_delay: 3000,
            consonant_transition_time: 0,
            consonant_on_time: 1000,
            consonant_off_time: 1000,
            consonant_volume: 0.1,
            consonant_position: 40,
            consonant_frequency: 2000.0,
            consonant_bandwidth: 3000.0
        };
        result.update_pan_positions();
        result
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
                let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, true);
                delay = self.add_transient_vowel(delay, *c, vowel_delay, vowel_transition_time);
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

            let mut final_consonants = Vec::new();
            let mut final_vowel = None;
            if let Some(note) = &self.current_note {
                final_vowel = Some(note.syllable.main_vowel);
                if note.syllable.final_vowels.len() > 0 {
                    final_vowel = Some(*note.syllable.final_vowels.last().unwrap());
                }
                final_consonants = note.syllable.final_consonants.clone();
            }
            if let Some(vowel) = final_vowel {
                let consonant = self.phonemes.get_consonant(*consonants.last().unwrap()).unwrap();
                let consonant_shape = self.phonemes.get_consonant_shape(&consonant, vowel);
                if consonant_shape.is_some() {
                    self.add_transition(delay, 1000, TransitionData::ShapeChange {
                        start_shape: self.shape_after_transitions.clone(),
                        end_shape: consonant_shape.unwrap().clone(),
                        start_nasal_coupling: self.nasal_coupling_after_transitions,
                        end_nasal_coupling: 0.0
                    });
                    delay += 1000;
                }
            }
            for c in final_consonants {
                let (delay_to_consonant, _delay_to_vowel) = self.add_consonant(delay, c, final_vowel, true);
                delay += delay_to_consonant;
            }
            let adjacent_vowel = match new_syllable.initial_vowels.first() {
                Some(v) => *v,
                None => new_syllable.main_vowel
            };
            for i in 0..new_syllable.initial_consonants.len() {
                let (delay_to_consonant, delay_to_vowel) = self.add_consonant(delay, new_syllable.initial_consonants[i], Some(adjacent_vowel), false);
                if i == new_syllable.initial_consonants.len()-1 {
                    delay += delay_to_vowel;
                }
                else {
                    delay += delay_to_consonant;
                }
            }
            self.add_transition(delay, 0, TransitionData::FrequencyChange {start_frequency: frequency, end_frequency: frequency});
        }

        // Play any initial vowels.

        for c in &new_syllable.initial_vowels {
            let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, false);
            delay = self.add_transient_vowel(delay, *c, vowel_delay, vowel_transition_time);
        }

        // Start the main vowel playing.

        let shape = self.phonemes.get_vowel_shape(new_syllable.main_vowel).unwrap();
        let nasal_coupling = self.phonemes.get_nasal_coupling(new_syllable.main_vowel);
        let amplification = self.phonemes.get_amplification(new_syllable.main_vowel);
        let transition_time = if self.current_note.is_some() || new_syllable.initial_vowels.len() > 0 || consonants.len() > 0 {self.vowel_transition_time} else {0};
        let start_shape = match self.consonants.last() {
            Some(c) => {
                let consonant_shape = self.phonemes.get_consonant_shape(c, new_syllable.main_vowel);
                if consonant_shape.is_some() {consonant_shape.unwrap().clone()} else {self.shape_after_transitions.clone()}
            },
            None => self.shape_after_transitions.clone()
        };
        self.add_transition(delay, transition_time, TransitionData::ShapeChange {
            start_shape: start_shape,
            end_shape: shape.clone(),
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: nasal_coupling
        });
        self.add_transition(delay, 2000, TransitionData::EnvelopeChange {
            start_envelope: self.envelope_after_transitions,
            end_envelope: amplification
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
        let mut final_vowel = None;
        if let Some(note) = &self.current_note {
            final_vowel = Some(note.syllable.main_vowel);
            for c in &note.syllable.final_vowels.clone() {
                let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, true);
                delay = self.add_transient_vowel(delay, *c, vowel_delay, vowel_transition_time);
                final_vowel = Some(*c);
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
            if let Some(vowel) = final_vowel {
                let consonant = self.phonemes.get_consonant(*consonants.last().unwrap()).unwrap();
                let consonant_shape = self.phonemes.get_consonant_shape(&consonant, vowel);
                if consonant_shape.is_some() {
                    self.add_transition(delay, 1500, TransitionData::ShapeChange {
                        start_shape: self.shape_after_transitions.clone(),
                        end_shape: consonant_shape.unwrap().clone(),
                        start_nasal_coupling: self.nasal_coupling_after_transitions,
                        end_nasal_coupling: 0.0
                    });
                    delay += 1500;
                }
            }
            for c in &consonants {
                let (delay_to_consonant, _delay_to_vowel) = self.add_consonant(delay, *c, final_vowel, true);
                delay += delay_to_consonant;
            }
        }
        self.current_note = None;
    }

    fn add_transient_vowel(&mut self, delay: i64, c: char, vowel_delay: i64, vowel_transition_time: i64) -> i64 {
        let shape = self.phonemes.get_vowel_shape(c).unwrap();
        let nasal_coupling = self.phonemes.get_nasal_coupling(c);
        let amplification = self.phonemes.get_amplification(c);
        self.add_transition(delay, vowel_transition_time, TransitionData::ShapeChange {
            start_shape: self.shape_after_transitions.clone(),
            end_shape: shape.clone(),
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: nasal_coupling
        });
        self.add_transition(delay, vowel_transition_time, TransitionData::EnvelopeChange {
            start_envelope: self.envelope_after_transitions,
            end_envelope: amplification
        });
        delay+vowel_delay+vowel_transition_time
    }

    fn add_consonant(&mut self, delay: i64, c: char, adjacent_vowel: Option<char>, is_final: bool) -> (i64, i64) {
        // let mut consonant = Consonant {
        //     sampa: c,
        //     start: self.step+delay,
        //     delay: self.consonant_delay,
        //     transition_time: self.vowel_transition_time,
        //     on_time: self.consonant_on_time,
        //     off_time: self.consonant_off_time,
        //     volume: self.consonant_volume,
        //     position: self.consonant_position,
        //     filter: ResonantFilter::new(48000, self.consonant_frequency, self.consonant_bandwidth)
        // };
        let mut consonant = self.phonemes.get_consonant(c).unwrap();
        // let delay = if is_final {delay+2000} else {delay};
        consonant.start = self.step+delay;
        let delay_to_consonant = consonant.delay+consonant.on_time+consonant.off_time;
        let delay_to_vowel = consonant.delay+consonant.transition_time;
        self.consonants.push(consonant);
        if let Some(vowel) = adjacent_vowel {
            let start_shape = self.phonemes.get_consonant_shape(&consonant, vowel).unwrap();
            let end_shape = if is_final {&start_shape} else {self.phonemes.get_vowel_shape(vowel).unwrap()};
            let nasal_coupling = self.phonemes.get_nasal_coupling(vowel);
            self.add_transition(delay, self.vowel_transition_time, TransitionData::ShapeChange {
                start_shape: start_shape.clone(),
                end_shape: end_shape.clone(),
                start_nasal_coupling: 0.0,
                end_nasal_coupling: nasal_coupling
            });
        }
        (delay_to_consonant, delay_to_vowel)
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

    pub fn generate(&mut self) -> (f32, f32) {
        if self.step%200 == 0 {
            self.process_messages();
            self.update_transitions();
        }
        self.step += 1;
        if self.envelope > 0.0 || self.consonants.len() != 0 {
            self.off_after_step = self.step+500;
        }
        let mut left = 0.0;
        let mut right = 0.0;
        if self.step < self.off_after_step {
            let mut consonant_finished = self.consonants.len() > 0;
            for i in 0..self.voices.len() {
                let mut consonant_noise = 0.0;
                let mut consonant_position = 0;
                if self.consonants.len() > 0 {
                    let consonant = &mut self.consonants[0];
                    if !consonant.mono || i == self.voices.len()/2 {
                        let j = self.step-consonant.start-self.consonant_delays[i];
                        if j > 0 {
                            consonant_position = consonant.position;
                            if j < consonant.on_time {
                                let volume = consonant.volume*(j as f32 / consonant.on_time as f32);
                                consonant_noise = volume*consonant.filter.process(2.0*self.random.get_uniform()-1.0);
                            }
                            else if j < consonant.on_time+consonant.off_time {
                                let k = j-consonant.on_time;
                                let volume = consonant.volume*((consonant.off_time-k) as f32 / consonant.off_time as f32);
                                consonant_noise = volume*consonant.filter.process(2.0*self.random.get_uniform()-1.0);
                            }
                        }
                        if consonant.mono {
                            consonant_noise *= (self.voices.len() as f32).sqrt();
                        }
                        if j < consonant.on_time+consonant.off_time {
                            consonant_finished = false;
                        }
                    }
                }
                let signal = self.voices[i].generate(self.step, consonant_noise, consonant_position);
                left += self.voice_pan[i].cos()*signal;
                right += self.voice_pan[i].sin()*signal;
            }
            if consonant_finished {
                self.consonants.remove(0);
                for i in 0..self.consonant_delays.len() {
                    if i != self.consonant_delays.len()/2 {
                        self.consonant_delays[i] = (self.random.get_int()%200) as i64;
                    }
                }
            }
        }
        (0.08*left, 0.08*right)
    }

    fn process_messages(&mut self) {
        loop {
            match self.message_receiver.try_recv() {
                Ok(message) => {
                    match message {
                        Message::NoteOn {syllable, note_index, velocity} => {
                            let _ = self.note_on(&syllable, note_index, velocity);
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
                        Message::SetStereoWidth {width} => {
                            self.stereo_width = width;
                            self.update_pan_positions();
                        }
                        Message::SetDelays {vowel_delay, vowel_transition_time, consonant_delay, consonant_transition_time} => {
                            self.vowel_delay = vowel_delay;
                            self.vowel_transition_time = vowel_transition_time;
                            self.consonant_delay = consonant_delay;
                            self.consonant_transition_time = consonant_transition_time;
                        }
                        Message::SetConsonants {on_time, off_time, volume, position, frequency, bandwidth} => {
                            self.consonant_on_time = on_time;
                            self.consonant_off_time = off_time;
                            self.consonant_volume = volume;
                            self.consonant_position = position;
                            self.consonant_frequency = frequency;
                            self.consonant_bandwidth = bandwidth;
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
                        let coupling = weight1*start_nasal_coupling + weight2*end_nasal_coupling;
                        let coupling = 8.0*coupling*coupling*coupling*coupling;
                        for voice in &mut self.voices {
                            let n = start_shape.len();
                            let mut shape = vec![0.0; n];
                            for i in 0..n {
                                shape[i] = weight1*start_shape[i] + weight2*end_shape[i];
                            }
                            voice.set_vocal_shape(&shape, coupling);
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
        let noise = 0.05*(1.0-self.volume)*(1.0-self.volume);
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

    fn update_pan_positions(&mut self) {
        let voice_count = self.voices.len();
        if voice_count == 1 {
            self.voice_pan[0] = 0.25*PI;
        }
        else {
            for i in 0..voice_count {
                self.voice_pan[i] = 0.5*PI*(0.5 + self.stereo_width*(i as f32 / (voice_count-1) as f32 - 0.5));
            }
        }
    }

    fn get_vowel_timing(&self, vowel: char, is_final: bool) -> (i64, i64) {
        if vowel == 'm' {
            if is_final {
                return (0, 1500);
            }
            return (3500, 1000);
        }
        if vowel == 'n' {
            if is_final {
                return (0, 1500);
            }
            return (1000, 1500);
        }
        if vowel == 'N' {
            return (1000, 1500);
        }
        (self.vowel_delay, self.vowel_transition_time)
    }
}