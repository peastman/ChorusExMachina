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

use crate::voice::Voice;
use crate::filter::Filter;
use crate::phonemes::{Consonant, Phonemes};
use crate::random::Random;
use crate::syllable::Syllable;
use crate::VoicePart;
use std::f32::consts::PI;
use std::sync::mpsc;

/// A message that can be sent to a Director.  Messages roughly correspond to MIDI events:
/// note on, note off, and various control channels.
pub enum Message {
    Reinitialize {voice_part: VoicePart, voice_count: usize},
    NoteOn {syllable: String, note_index: i32, velocity: f32},
    NoteOff,
    SetVolume {volume: f32},
    SetPitchBend {semitones: f32},
    SetVibrato {vibrato: f32},
    SetIntensity {intensity: f32},
    SetBrightness {brightness: f32},
    SetConsonantVolume {volume: f32},
    SetAttackRate {attack: f32},
    SetAccent {accent: bool},
    SetStereoWidth {width: f32},
    SetDelays {vowel_delay: i64, vowel_transition_time: i64, consonant_delay: i64, consonant_transition_time: i64},
    SetConsonants {on_time: i64, off_time: i64, volume: f32, position: usize, frequency: f32, bandwidth: f32},
    SetRandomize {randomize: f32}
}

/// A Transition describes some type of continuous change to the voices.  It specifies the time
/// interval (in step indices) over which the change takes place.  The details of what is
/// changing are specified by the TransitionData.
struct Transition {
    start: i64,
    end: i64,
    data: TransitionData
}

/// A TransitionData is contained in a Transition.  It specifies what aspect of the voices is
/// changing, and what values it is changing between.
enum TransitionData {
    EnvelopeChange {start_envelope: f32, end_envelope: f32},
    ShapeChange {start_shape: Vec<Vec<f32>>, end_shape: Vec<Vec<f32>>, start_nasal_coupling: f32, end_nasal_coupling: f32},
    FrequencyChange {start_frequency: f32, end_frequency: f32}
}

/// A note that is being sung.  It is described by the standard MIDI properties (note index
/// and velocity), as well as the syllable to sing it on.
struct Note {
    syllable: Syllable,
    note_index: i32
}

/// This is the main class you interact with when synthesizing audio.  A Director controls a set
/// of Voices, all of the same voice part, that sing in unison.  It handles all details of
/// pronunciation and expression to make them sing the requested notes and syllables.
///
/// When creating a Director with new(), you provide a Receiver<Message> that has been created
/// with mpsc::channel().  You control it by sending messages from the corresponding Sender.
/// The only method you call directly on it is generate(), which is used to generate samples.
/// This design allows control and generation to happen on different threads.
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
    vibrato: f32,
    intensity: f32,
    brightness: f32,
    consonant_volume: f32,
    attack_rate: f32,
    accent: bool,
    off_after_step: i64,
    shape_after_transitions: Vec<Vec<f32>>,
    nasal_coupling_after_transitions: f32,
    envelope_after_transitions: f32,
    frequency_after_transitions: f32,
    message_receiver: mpsc::Receiver<Message>,
    stereo_width: f32,
    voice_pan: Vec<f32>,
    dark_shape: Vec<f32>,
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
    randomize: f32
}

impl Director {
    pub fn new(voice_part: VoicePart, voice_count: usize, message_receiver: mpsc::Receiver<Message>) -> Self {
        let mut result = Self {
            voices: vec![],
            voice_part: voice_part.clone(),
            lowest_note: 0,
            highest_note: 0,
            phonemes: Phonemes::new(voice_part),
            random: Random::new(),
            step: 0,
            transitions: vec![],
            current_note: None,
            consonants: vec![],
            consonant_delays: vec![],
            volume: 1.0,
            envelope: 0.0,
            frequency: 1.0,
            bend: 1.0,
            vibrato: 0.4,
            intensity: 0.5,
            brightness: 1.0,
            consonant_volume: 0.5,
            attack_rate: 0.8,
            accent: false,
            off_after_step: 0,
            shape_after_transitions: vec![],
            nasal_coupling_after_transitions: 0.0,
            envelope_after_transitions: 0.0,
            frequency_after_transitions: 0.0,
            message_receiver: message_receiver,
            stereo_width: 0.3,
            voice_pan: vec![],
            dark_shape: vec![],
            vowel_delay: 0,
            vowel_transition_time: 3500,
            consonant_delay: 3000,
            consonant_transition_time: 0,
            consonant_on_time: 1000,
            consonant_off_time: 1000,
            consonant_volume2: 0.1,
            consonant_position: 40,
            consonant_frequency: 2000.0,
            consonant_bandwidth: 3000.0,
            randomize: 0.1
        };
        result.initialize_voices(voice_part, voice_count);
        result
    }

    fn initialize_voices(&mut self, voice_part: VoicePart, voice_count: usize) {
        self.voice_part = voice_part.clone();
        self.voices.clear();
        for i in 0..voice_count {
            self.voices.push(Voice::new(voice_part, i));
        }
        self.phonemes = Phonemes::new(voice_part);
        self.transitions.clear();
        self.current_note = None;
        self.consonants.clear();
        self.consonant_delays = Vec::new();
        for _ in 0..voice_count {
            self.consonant_delays.push((self.random.get_int()%1000) as i64);
        }
        self.voice_pan = vec![0.0; voice_count];
        self.envelope = 0.0;
        self.bend = 1.0;
        self.nasal_coupling_after_transitions = 0.0;
        self.envelope_after_transitions = 0.0;
        self.frequency_after_transitions = 0.0;
        let vocal_length;
        match voice_part {
            VoicePart::Soprano => {
                vocal_length = 42;
                self.lowest_note = 57;
                self.highest_note = 88;
            }
            VoicePart::Alto => {
                vocal_length = 45;
                self.lowest_note = 48;
                self.highest_note = 79;
            }
            VoicePart::Tenor => {
                vocal_length = 48;
                self.lowest_note = 43;
                self.highest_note = 72;
            }
            VoicePart::Bass => {
                vocal_length = 50;
                self.lowest_note = 36;
                self.highest_note = 67;
            }
        }
        self.shape_after_transitions = vec![vec![0.0; vocal_length]; voice_count];
        self.dark_shape = self.phonemes.get_vowel_shape('9').unwrap().clone();
        self.update_pan_positions();
        self.update_vibrato();
        self.update_volume();
        self.update_frequency();
        self.update_sound();
    }

    /// Start singing a new note.
    fn note_on(&mut self, syllable: &str, note_index: i32, velocity: f32) -> Result<(), &'static str> {
        // If the note index is outside the range of this voice part, just stop the current
        // note and exit.

        if note_index < self.lowest_note || note_index > self.highest_note {
            if self.current_note.is_some() {
                self.note_off(false);
            }
            return Ok(());
        }

        // Prepare for playing the note.

        let new_syllable = Syllable::build(syllable)?;
        if let Some(note) = &self.current_note {
            if note.syllable.final_consonants.len() > 0 || new_syllable.initial_consonants.len() > 0 {
                // Playing legato isn't possible, since there are consonants between the vowels.
                // Finish the current note.

                self.note_off(true);
            }
        }
        let frequency = 440.0 * f32::powf(2.0, (note_index-69) as f32/12.0);
        let mut delay = 0;
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }
        let mut prev_vowel = None;
        if let Some(note) = &self.current_note {
            // Play any final vowels from the previous note.

            let current_note_index = note.note_index;
            prev_vowel = Some(note.syllable.main_vowel);
            for c in &note.syllable.final_vowels.clone() {
                let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, true);
                delay = self.add_transient_vowel(delay, prev_vowel, *c, vowel_delay, vowel_transition_time, true);
                prev_vowel = Some(*c);
            }

            // Smoothly transition between the two notes.

            let transition_time = (1100 + 270*(current_note_index-note_index).abs()) as i64;
            self.add_transition(delay, transition_time, TransitionData::FrequencyChange {start_frequency: self.frequency_after_transitions, end_frequency: frequency});
            let min_envelope = f32::powf(0.9, (current_note_index-note_index).abs() as f32);
            self.add_transition(delay, transition_time/2, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: min_envelope});
            self.add_transition(delay+transition_time/2, transition_time/2, TransitionData::EnvelopeChange {start_envelope: min_envelope, end_envelope: 1.0});
            delay += transition_time;
        }
        else {
            // Play any initial consonants from the new note.

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

            // Set the frequency of the new note.

            self.add_transition(delay, 0, TransitionData::FrequencyChange {start_frequency: frequency, end_frequency: frequency});
        }

        // Play any initial vowels.

        for c in &new_syllable.initial_vowels {
            let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, false);
            delay = self.add_transient_vowel(delay, prev_vowel, *c, vowel_delay, vowel_transition_time, false);
            prev_vowel = Some(*c);
        }

        // Start the main vowel playing.

        let shape = self.phonemes.get_vowel_shape(new_syllable.main_vowel).unwrap().clone();
        let nasal_coupling = self.phonemes.get_nasal_coupling(new_syllable.main_vowel);
        let transition_time = if self.current_note.is_some() || new_syllable.initial_vowels.len() > 0 || new_syllable.initial_consonants.len() > 0 {self.vowel_transition_time} else {0};
        if prev_vowel.is_some() {
            self.add_vowel_transition(delay, prev_vowel.unwrap(), new_syllable.main_vowel, self.vowel_transition_time);
        }
        else {
            if let Some(c) = self.consonants.last() {
                if new_syllable.initial_vowels.len() == 0 {
                    let consonant_shape = self.phonemes.get_consonant_shape(c, new_syllable.main_vowel);
                    if consonant_shape.is_some() {
                        self.add_shape_transition(delay, 0, consonant_shape.unwrap().clone(), self.nasal_coupling_after_transitions);
                    }
                }
            };
            self.add_shape_transition(delay, transition_time, shape, nasal_coupling)
        }

        // Adjust the envelope for the new note.  If accent is enabled, overshoot it then come back down.

        let amplification = self.phonemes.get_amplification(new_syllable.main_vowel);
        let max_amplitude = if self.accent {amplification*(1.0+2.5*velocity)} else {amplification};
        let attack_time = 1000+(10000.0*(1.0-self.attack_rate)) as i64;
        self.add_transition(delay, attack_time, TransitionData::EnvelopeChange {
            start_envelope: self.envelope_after_transitions,
            end_envelope: max_amplitude
        });
        if self.accent {
            self.add_transition(delay+attack_time, 4000, TransitionData::EnvelopeChange {
                start_envelope: max_amplitude,
                end_envelope: amplification
            });
        }

        // Record the note we're now playing.

        let note = Note {
            syllable: new_syllable,
            note_index: note_index
        };
        self.current_note = Some(note);
        self.update_sound();
        Ok(())
    }

    /// End the current note.  Because this is a monophonic instrument, note_on() automatically
    /// ends the current note as well.
    fn note_off(&mut self, legato: bool) {
        let mut delay = 0;
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }

        // Play any final vowels.

        let mut final_vowel = None;
        if let Some(note) = &self.current_note {
            final_vowel = Some(note.syllable.main_vowel);
            for c in &note.syllable.final_vowels.clone() {
                let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, true);
                delay = self.add_transient_vowel(delay, final_vowel, *c, vowel_delay, vowel_transition_time, true);
                final_vowel = Some(*c);
            }
        }

        // Determine how quickly to stop the sound.  It depends on whether the first final consonant
        // is voiced.

        let stop_envelope_time = delay;
        let mut off_time = if legato {1500} else {2000};

        // Play any final consonants.

        let mut consonants = Vec::new();
        if let Some(note) = &self.current_note {
            for c in &note.syllable.final_consonants {
                consonants.push(*c);
            }
        }
        if consonants.len() > 0 {
            if let Some(vowel) = final_vowel {
                let consonant = self.phonemes.get_consonant(*consonants.last().unwrap(), true).unwrap();
                let consonant_shape = self.phonemes.get_consonant_shape(&consonant, vowel);
                if consonant_shape.is_some() {
                    self.add_shape_transition(delay, 2000, consonant_shape.unwrap().clone(), 0.0);
                    delay += 2000;
                }
            }
            let first_consonant = self.phonemes.get_consonant(consonants[0], true).unwrap();
            if first_consonant.voiced {
                off_time = off_time.max(first_consonant.delay+first_consonant.transition_time+first_consonant.on_time);
            }
            for c in &consonants {
                let (delay_to_consonant, _delay_to_vowel) = self.add_consonant(delay, *c, final_vowel, true);
                delay += delay_to_consonant;
            }
        }

        // Smoothly stop the sound.

        self.add_transition(stop_envelope_time, off_time, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: 0.0});
        self.current_note = None;
    }

    /// Add the Transitions to play a transient vowel (an initial or final vowel that sounds
    /// only briefly).
    fn add_transient_vowel(&mut self, delay: i64, prev_vowel: Option<char>, c: char, vowel_delay: i64, vowel_transition_time: i64, is_final: bool) -> i64 {
        if prev_vowel.is_some() {
            self.add_vowel_transition(delay, prev_vowel.unwrap(), c, vowel_transition_time);
        }
        else {
            let shape = self.phonemes.get_vowel_shape(c).unwrap();
            let nasal_coupling = self.phonemes.get_nasal_coupling(c);
            self.add_shape_transition(delay, vowel_transition_time, shape.clone(), nasal_coupling);
        }
        let scale = if is_final {0.5} else {0.7};
        let amplification = scale*self.phonemes.get_amplification(c);
        self.add_transition(delay, vowel_transition_time, TransitionData::EnvelopeChange {
            start_envelope: self.envelope_after_transitions,
            end_envelope: amplification
        });
        delay+vowel_delay+vowel_transition_time
    }

    /// Add the Transitions to smoothly change the vocal tract shape between two vowels.
    fn add_vowel_transition(&mut self, delay: i64, vowel1: char, vowel2: char, vowel_transition_time: i64) {
        let shape = self.phonemes.get_vowel_shape(vowel2).unwrap().clone();
        let nasal_coupling = self.phonemes.get_nasal_coupling(vowel2);
        if let Some(intermediate_shape) = self.phonemes.get_intermediate_shape(vowel1, vowel2) {
            let intermediate_coupling = 0.5*(self.nasal_coupling_after_transitions+nasal_coupling);
            self.add_shape_transition(delay, vowel_transition_time/2, intermediate_shape.clone(), intermediate_coupling);
            self.add_shape_transition(delay+vowel_transition_time/2, vowel_transition_time/2, shape, nasal_coupling);
        }
        else {
            self.add_shape_transition(delay, vowel_transition_time, shape, nasal_coupling);
        }
    }

    /// Play a consonant.  This adds a Consonant to the queue, and if necessary also adds a
    /// Transition to control the vocal tract shape appropriately.
    fn add_consonant(&mut self, delay: i64, c: char, adjacent_vowel: Option<char>, is_final: bool) -> (i64, i64) {
        let mut consonant = self.phonemes.get_consonant(c, is_final).unwrap();
        consonant.start = self.step+delay;
        consonant.volume *= 2.0*self.consonant_volume;
        if is_final {
            consonant.off_time = (consonant.off_time as f32 * 0.8) as i64;
        }
        let delay_to_consonant = consonant.delay+consonant.on_time+consonant.off_time;
        let delay_to_vowel = consonant.delay;
        if !consonant.mono {
            consonant.volume /= (self.voices.len() as f32).sqrt();
        }
        self.consonants.push(consonant);
        if let Some(vowel) = adjacent_vowel {
            let start_shape = self.phonemes.get_consonant_shape(&consonant, vowel).unwrap().clone();
            let end_shape = (if is_final {&start_shape} else {self.phonemes.get_vowel_shape(vowel).unwrap()}).clone();
            let nasal_coupling = self.phonemes.get_nasal_coupling(vowel);
            self.add_shape_transition(delay, 1000, start_shape, 0.0);
            self.add_shape_transition(delay+1000, self.vowel_transition_time, end_shape, nasal_coupling);
        }
        (delay_to_consonant, delay_to_vowel)
    }

    /// Add a Transition to the queue.
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

    /// Add a ShapeChange transition to the queue.
    fn add_shape_transition(&mut self, delay: i64, duration: i64, mut end_shape: Vec<f32>, end_nasal_coupling: f32) {
        if end_nasal_coupling == 0.0 && self.brightness < 1.0 {
            let blend = (1.0-self.brightness)*0.2;
            for i in 0..end_shape.len() {
                end_shape[i] = (1.0-blend)*end_shape[i] + blend*self.dark_shape[i];
            }
        }
        let mut end_shapes = vec![end_shape; self.voices.len()];
        for shape in &mut end_shapes {
            for x in shape {
                *x *= 0.9 + 0.2*self.random.get_uniform();
            }
        }
        self.add_transition(delay, duration, TransitionData::ShapeChange {
            start_shape: self.shape_after_transitions.clone(),
            end_shape: end_shapes,
            start_nasal_coupling: self.nasal_coupling_after_transitions,
            end_nasal_coupling: end_nasal_coupling
        });
    }

    /// This is called repeated to generate audio data.  Each generates the two channels
    /// (left, right) for the next sample.
    pub fn generate(&mut self) -> (f32, f32) {
        // Deal with the queues of Messages and Transitions.  This only needs to be done occassionally.

        if self.step%200 == 0 {
            self.process_messages();
            self.update_transitions();
        }

        // If there has been no glottal excitation and no consonant for a while, we can just
        // return without doing any work.

        self.step += 1;
        if self.envelope > 0.0 || self.consonants.len() != 0 {
            self.off_after_step = self.step+500;
        }
        let mut left = 0.0;
        let mut right = 0.0;
        if self.step < self.off_after_step {
            let mut consonant_finished = self.consonants.len() > 0;

            // Loop over Voices and generate audio for each one.

            for i in 0..self.voices.len() {
                // If a Consonant is being sung, generate the noise signal for it.

                let mut consonant_noise = 0.0;
                let mut consonant_position = 0;
                if self.consonants.len() > 0 {
                    let consonant = &mut self.consonants[0];

                    // Mono consonants are only sung by one voice (the one panned to the center).
                    // Others are sung by ever voice.

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

                // Generate audio for the voice, injecting the consonant noise if appropriate.

                let signal = self.voices[i].generate(self.step, consonant_noise, consonant_position);
                left += self.voice_pan[i].cos()*signal;
                right += self.voice_pan[i].sin()*signal;
            }
            if consonant_finished {
                self.consonants.remove(0);
                for i in 0..self.consonant_delays.len() {
                    if i != self.consonant_delays.len()/2 {
                        self.consonant_delays[i] = (self.random.get_int()%1000) as i64;
                    }
                }
            }
        }
        (0.08*left, 0.08*right)
    }

    /// This is called occasionally by generate().  It processes any Messages that have been
    /// received since the last call.
    fn process_messages(&mut self) {
        loop {
            match self.message_receiver.try_recv() {
                Ok(message) => {
                    match message {
                        Message::Reinitialize {voice_part, voice_count} => {
                            self.initialize_voices(voice_part, voice_count);
                        }
                        Message::NoteOn {syllable, note_index, velocity} => {
                            let _ = self.note_on(&syllable, note_index, velocity);
                        }
                        Message::NoteOff => {
                            self.note_off(false);
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
                        Message::SetVibrato {vibrato} => {
                            self.vibrato = vibrato;
                            self.update_vibrato();
                        }
                        Message::SetIntensity {intensity} => {
                            self.intensity = intensity;
                            self.update_sound();
                        }
                        Message::SetBrightness {brightness} => {
                            self.brightness = brightness;
                        }
                        Message::SetConsonantVolume {volume} => {
                            self.consonant_volume = volume;
                        }
                        Message::SetAttackRate {attack} => {
                            self.attack_rate = attack;
                        }
                        Message::SetAccent {accent} => {
                            self.accent = accent;
                        }
                        Message::SetStereoWidth {width} => {
                            self.stereo_width = width;
                            self.update_pan_positions();
                        }
                        Message::SetDelays {vowel_delay, vowel_transition_time, consonant_delay, consonant_transition_time} => {
                            // This message is only used for develoment.
                            self.vowel_delay = vowel_delay;
                            self.vowel_transition_time = vowel_transition_time;
                            self.consonant_delay = consonant_delay;
                            self.consonant_transition_time = consonant_transition_time;
                        }
                        Message::SetConsonants {on_time, off_time, volume, position, frequency, bandwidth} => {
                            // This message is only used for develoment.
                            self.consonant_on_time = on_time;
                            self.consonant_off_time = off_time;
                            self.consonant_volume2 = volume;
                            self.consonant_position = position;
                            self.consonant_frequency = frequency;
                            self.consonant_bandwidth = bandwidth;
                        }
                        Message::SetRandomize {randomize} => {
                            self.randomize = randomize;
                        }
                    }
                }
                Err(_) => {
                    break;
                }
            }
        }
    }

    /// This is called occasionally by generate().  It processes any Transitions in the queue,
    /// updating the voices as appropriate.
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
                        for i in 0..self.voices.len() {
                            let n = start_shape[i].len();
                            let mut shape = vec![0.0; n];
                            for j in 0..n {
                                shape[j] = weight1*start_shape[i][j] + weight2*end_shape[i][j];
                            }
                            self.voices[i].set_vocal_shape(&shape, coupling);
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

    /// Update the volumes of all Voices.  This is called whenever the Director's volume or
    /// envelope is changed.
    fn update_volume(&mut self) {
        let actual_volume = 0.1+0.9*self.volume;
        for voice in &mut self.voices {
            voice.set_volume(actual_volume*self.envelope);
        }
    }

    /// Update the frequencies of all Voices.  This is called whenever the Director's frequency or
    /// pitch bend is changed.
    fn update_frequency(&mut self) {
        for voice in &mut self.voices {
            voice.set_frequency(self.frequency*self.bend);
        }
    }

    /// Update the vibrato of all Voices.  This is called whenever the Director's vibrato is changed.
    fn update_vibrato(&mut self) {
        let amplitude = 0.04*(self.vibrato+0.1);
        let n = self.voices.len();
        for (i, voice) in &mut self.voices.iter_mut().enumerate() {
            if n < 4 {
                voice.set_vibrato_amplitude(amplitude*(1.0-0.25*i as f32));
            }
            else {
                voice.set_vibrato_amplitude(amplitude*(1.0-0.5*(i as f32)/((n-1) as f32)));
            }
        }
    }

    /// Update Rd and noise amplitude for all voices.  They depend on the volume and the note
    /// being played.
    fn update_sound(&mut self) {
        let noise = 0.05*(1.0-self.volume)*(1.0-self.volume);
        for voice in &mut self.voices {
            voice.set_noise(noise);
        }
        if let Some(note) = &self.current_note {
            let x = (self.highest_note-note.note_index) as f32 / (self.highest_note-self.lowest_note) as f32;
            let rd = 1.6 + 0.4*x - 0.2*self.volume - (self.intensity-0.5);
            for (i, voice) in &mut self.voices.iter_mut().enumerate() {
                voice.set_rd(rd + 0.1*(i%4) as f32);
            }
        }
    }

    /// Update the position each voice is panned to.
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

    /// Get the timing parameters (delay, transition time) for a transient vowel.
    fn get_vowel_timing(&self, vowel: char, is_final: bool) -> (i64, i64) {
        if vowel == 'm' {
            if is_final {
                return (0, 1500);
            }
            return (1000, 1500);
        }
        if vowel == 'n' {
            if is_final {
                return (0, 1500);
            }
            return (1000, 2500);
        }
        if vowel == 'N' {
            return (1000, 1500);
        }
        if vowel == '3' {
            return (self.vowel_delay, 2000)
        }
        if is_final {
            return (self.vowel_delay, 2000)
        }
        (self.vowel_delay, self.vowel_transition_time)
    }
}