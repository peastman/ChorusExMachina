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
    NoteOn {syllable: String, note_index: i32, velocity: f32, continue_syllable: bool},
    NoteOff,
    SetVolume {volume: f32},
    SetPitchBend {semitones: f32},
    SetVibrato {vibrato: f32},
    SetIntensity {intensity: f32},
    SetBrightness {brightness: f32},
    SetConsonantVolume {volume: f32},
    SetAttackRate {attack: f32},
    SetReleaseRate {release: f32},
    SetAccent {accent: bool},
    SetStereoWidth {width: f32},
    SetMinVowelStartTime {samples: i64},
    SetMaxVoiceDelay {max_delay: i64},
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
    high_blend_note: i32,
    high_blend_fraction: f32,
    phonemes: Phonemes,
    random: Random,
    step: i64,
    transitions: Vec<Transition>,
    current_note: Option<Note>,
    consonants: Vec<Consonant>,
    max_voice_delay: i64,
    voice_delays: Vec<i64>,
    volume: f32,
    envelope: Vec<f32>,
    frequency: Vec<f32>,
    bend: f32,
    vibrato: f32,
    intensity: f32,
    brightness: f32,
    consonant_volume: f32,
    attack_rate: f32,
    release_rate: f32,
    accent: bool,
    min_vowel_start: i64,
    off_after_step: i64,
    shape_after_transitions: Vec<Vec<f32>>,
    nasal_coupling_after_transitions: f32,
    envelope_after_transitions: f32,
    frequency_after_transitions: f32,
    message_receiver: mpsc::Receiver<Message>,
    stereo_width: f32,
    voice_pan: Vec<f32>,
    dark_shape: Vec<f32>,
    high_shape: Vec<f32>,
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
            high_blend_note: 0,
            high_blend_fraction: 0.0,
            phonemes: Phonemes::new(voice_part),
            random: Random::new(),
            step: 0,
            transitions: vec![],
            current_note: None,
            consonants: vec![],
            max_voice_delay: 2000,
            voice_delays: vec![],
            volume: 1.0,
            envelope: vec![],
            frequency: vec![],
            bend: 1.0,
            vibrato: 0.4,
            intensity: 0.5,
            brightness: 1.0,
            consonant_volume: 0.5,
            attack_rate: 0.8,
            release_rate: 0.5,
            accent: false,
            min_vowel_start: 0,
            off_after_step: 0,
            shape_after_transitions: vec![],
            nasal_coupling_after_transitions: 0.0,
            envelope_after_transitions: 0.0,
            frequency_after_transitions: 0.0,
            message_receiver: message_receiver,
            stereo_width: 0.3,
            voice_pan: vec![],
            dark_shape: vec![],
            high_shape: vec![],
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

    /// Initialize the set of voices controlled by this Director.  This is called when it is first
    /// created, and again whenever a Reinitialize message is received.
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
        self.voice_delays = vec![0; voice_count];
        self.voice_pan = vec![0.0; voice_count];
        self.envelope = vec![0.0; voice_count];
        self.frequency = vec![0.0; voice_count];
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
                self.high_blend_note = 72;
                self.high_blend_fraction = 0.3;
            }
            VoicePart::Alto => {
                vocal_length = 45;
                self.lowest_note = 48;
                self.highest_note = 79;
                self.high_blend_note = 72;
                self.high_blend_fraction = 0.15;
            }
            VoicePart::Tenor => {
                vocal_length = 48;
                self.lowest_note = 43;
                self.highest_note = 72;
                self.high_blend_note = 64;
                self.high_blend_fraction = 0.1;
            }
            VoicePart::Bass => {
                vocal_length = 52;
                self.lowest_note = 36;
                self.highest_note = 67;
                self.high_blend_note = 60;
                self.high_blend_fraction = 0.1;
            }
        }
        self.shape_after_transitions = vec![vec![0.0; vocal_length]; voice_count];
        self.dark_shape = self.phonemes.get_vowel_shape('o').unwrap().clone();
        self.high_shape = self.phonemes.get_vowel_shape('A').unwrap().clone();
        for i in 0..self.high_shape.len() {
            self.high_shape[i] = 0.7*self.high_shape[i] + 0.3*self.dark_shape[i];
        }
        self.update_pan_positions();
        self.update_vibrato();
        self.update_volume();
        self.update_frequency();
        self.update_sound();
        self.update_voice_delays();
    }

    /// Start singing a new note.
    fn note_on(&mut self, syllable: &str, note_index: i32, velocity: f32, continue_syllable: bool) -> Result<(), String> {
        // If the note index is outside the range of this voice part, just stop the current
        // note and exit.

        if note_index < self.lowest_note || note_index > self.highest_note {
            if self.current_note.is_some() {
                self.note_off(false, false);
            }
            return Ok(());
        }

        // Prepare for playing the note.

        let num_transitions = self.transitions.len();
        let num_consonants = self.consonants.len();
        let new_syllable = Syllable::build(syllable)?;
        let mut delay_for_consonants = false;
        let has_current_note = self.current_note.is_some();
        let mut continuous = false;
        let mut sustain = new_syllable.initial_consonants.iter().all(|&c| self.phonemes.is_voiced_consonant(c));
        if let Some(note) = &self.current_note {
            if continue_syllable && note.syllable.main_vowel == new_syllable.main_vowel {
                // Treat the previous and new notes as a single syllable, continuing on the same vowel.

                continuous = true;
            }
            else if note.syllable.final_consonants.len() > 0 || new_syllable.initial_consonants.len() > 0 {
                // Smoothly blending the notes isn't possible, since there are consonants between the vowels.
                // Finish the current note.

                if note.syllable.final_consonants.len()+new_syllable.initial_consonants.len() > 1 {
                    delay_for_consonants = true;
                }
                if note.syllable.final_consonants.len() == 1 && !self.phonemes.is_voiced_consonant(note.syllable.final_consonants[0]) {
                    delay_for_consonants = true;
                }
                if new_syllable.initial_consonants.len() == 1 && !self.phonemes.is_voiced_consonant(new_syllable.initial_consonants[0]) {
                    delay_for_consonants = true;
                }
                sustain &= note.syllable.final_consonants.iter().all(|&c| self.phonemes.is_voiced_consonant(c));
                self.note_off(true, sustain);
            }
        }
        let frequency = 440.0 * f32::powf(2.0, (note_index-69) as f32/12.0);
        let mut delay = 0;
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }
        if delay_for_consonants {
            for consonant in &self.consonants {
                delay = i64::max(delay, consonant.start+consonant.on_time+consonant.off_time-self.step);
            }
        }
        let mut prev_vowel = None;
        let mut envelope_offset = 0;
        if let Some(note) = &self.current_note {
            let current_note_index = note.note_index;
            if !continuous {
                // Play any final vowels from the previous note.

                prev_vowel = Some(note.syllable.main_vowel);
                for c in &note.syllable.final_vowels.clone() {
                    let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, true);
                    delay = self.add_transient_vowel(delay, 0, prev_vowel, *c, vowel_delay, vowel_transition_time, true, true, current_note_index);
                    prev_vowel = Some(*c);
                }
            }

            // Smoothly transition between the two notes.  The time and envelope shape depend both on how
            // large a jump we're making and on what vowel we're heading toward.

            let transition_time = (1000 + 150*(current_note_index-note_index).abs()) as i64;
            self.add_transition(delay, transition_time, TransitionData::FrequencyChange {start_frequency: self.frequency_after_transitions, end_frequency: frequency});
            let min_envelope = self.envelope_after_transitions*f32::powf(0.85, (current_note_index-note_index).abs() as f32);
            let first_vowel;
            if !continuous && new_syllable.initial_vowels.len() > 0 {
                first_vowel = new_syllable.initial_vowels[0];
            }
            else {
                first_vowel = new_syllable.main_vowel;
            }
            let mut final_envelope = f32::min(1.0, self.phonemes.get_amplification(first_vowel));
            if current_note_index > note_index {
                final_envelope = 0.5*(final_envelope+min_envelope);
            }
            self.add_transition(delay, transition_time/2, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: min_envelope});
            self.add_transition(delay+transition_time/2, transition_time/2, TransitionData::EnvelopeChange {start_envelope: min_envelope, end_envelope: final_envelope});
            delay += transition_time;
        }
        else {
            // Set the frequency of the new note.

            self.add_transition(delay, 1000, TransitionData::FrequencyChange {start_frequency: frequency, end_frequency: frequency});

            // Play any initial consonants from the new note.

            let adjacent_vowel = match new_syllable.initial_vowels.first() {
                Some(v) => *v,
                None => new_syllable.main_vowel
            };
            for i in 0..new_syllable.initial_consonants.len() {
                let mut time_scale = if has_current_note {0.8} else {1.0};
                if new_syllable.initial_consonants.len() > 1 {
                    time_scale *= 0.8;
                }
                let (delay_to_consonant, delay_to_vowel, offset) = self.add_consonant(delay, new_syllable.initial_consonants[i], Some(adjacent_vowel), false, note_index, time_scale, 1.0);
                envelope_offset = offset;
                if i == new_syllable.initial_consonants.len()-1 {
                    delay += delay_to_vowel;
                }
                else {
                    delay += delay_to_consonant;
                }
            }
        }

        // Define a function that updates the start times of any transitions or consonants that were just added,
        // so as to make the vowel start at the right time.  This might be called either before or after the first
        // initial vowel, depending on what it is.

        let update_starts = |director: &mut Director, delay: &mut i64, has_updated_starts: &mut bool| {
            if *delay < director.min_vowel_start {
                let offset = director.min_vowel_start-*delay;
                for i in num_transitions..director.transitions.len() {
                    director.transitions[i].start += offset;
                    director.transitions[i].end += offset;
                }
                for i in num_consonants..director.consonants.len() {
                    director.consonants[i].start += offset;
                }
                *delay = director.min_vowel_start;
            }
            *has_updated_starts = true;
        };

        // Play any initial vowels.

        let mut has_updated_starts = false;
        let mut attack_time = if new_syllable.initial_consonants.len() == 0 {1000+(10000.0*(1.0-self.attack_rate)) as i64} else {0};
        if !continuous {
            for c in &new_syllable.initial_vowels {
                if !has_updated_starts && *c != 'l' && *c != 'm' && *c != 'n' {
                    update_starts(self, &mut delay, &mut has_updated_starts);
                }
                let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, false);
                delay = self.add_transient_vowel(delay, envelope_offset, prev_vowel, *c, vowel_delay, vowel_transition_time.max(attack_time), false, has_current_note, note_index);
                attack_time = 0;
                prev_vowel = Some(*c);
            }
        }
        if !has_updated_starts {
            update_starts(self, &mut delay, &mut has_updated_starts);
        }

        // Start the main vowel playing.

        let shape = self.phonemes.get_vowel_shape(new_syllable.main_vowel).unwrap().clone();
        let nasal_coupling = self.phonemes.get_nasal_coupling(new_syllable.main_vowel);
        let transition_time = if has_current_note || new_syllable.initial_vowels.len() > 0 || new_syllable.initial_consonants.len() > 0 {self.vowel_transition_time} else {0};
        if prev_vowel.is_some() {
            self.add_vowel_transition(delay, prev_vowel.unwrap(), new_syllable.main_vowel, self.vowel_transition_time, note_index);
        }
        else {
            self.add_shape_transition(delay, transition_time, shape, nasal_coupling, note_index, true)
        }

        // Adjust the envelope for the new note.  If accent is enabled, overshoot it then come back down.

        let amplification = self.phonemes.get_amplification(new_syllable.main_vowel);
        let max_amplitude = if self.accent {amplification*(1.0+2.5*velocity)} else {amplification};
        let (_vowel_delay, vowel_transition_time) = self.get_vowel_timing(new_syllable.main_vowel, false);
        attack_time = vowel_transition_time.max(attack_time);
        self.add_transition(delay-envelope_offset, attack_time, TransitionData::EnvelopeChange {
            start_envelope: self.envelope_after_transitions,
            end_envelope: max_amplitude
        });
        if self.accent {
            self.add_transition(delay-envelope_offset+attack_time, 4000, TransitionData::EnvelopeChange {
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
    fn note_off(&mut self, legato: bool, sustain: bool) {
        let mut delay = 0;
        let num_transitions = self.transitions.len();
        for transition in &self.transitions {
            delay = i64::max(delay, transition.end-self.step);
        }

        // Play any final vowels.

        let mut final_vowel = None;
        let mut note_index = -1;
        if let Some(note) = &self.current_note {
            note_index = note.note_index;
            final_vowel = Some(note.syllable.main_vowel);
            for c in &note.syllable.final_vowels.clone() {
                let (vowel_delay, vowel_transition_time) = self.get_vowel_timing(*c, true);
                delay = self.add_transient_vowel(delay, 0, final_vowel, *c, vowel_delay, vowel_transition_time, true, legato, note_index);
                final_vowel = Some(*c);
            }
        }

        // Update the start times of any vowels we just added so the final consonants will be
        // right on the beat.

        if !legato && delay < self.min_vowel_start {
            let offset = self.min_vowel_start-delay;
            for i in num_transitions..self.transitions.len() {
                self.transitions[i].start += offset;
                self.transitions[i].end += offset;
            }
            delay = self.min_vowel_start;
        }

        // Determine how quickly to stop the sound.  This may get modified if the first final consonant
        // is voiced.

        let mut stop_envelope_time = delay;
        let mut off_time = if legato {1500} else {1000 + (6000.0*(1.0-self.release_rate)) as i64};

        // Play any final consonants.

        let mut consonants = Vec::new();
        if let Some(note) = &self.current_note {
            for c in &note.syllable.final_consonants {
                consonants.push(*c);
            }
        }
        if consonants.len() > 0 {
            let first_consonant = self.phonemes.get_consonant(consonants[0], final_vowel, true, 1.0).unwrap();
            if first_consonant.voiced {
                stop_envelope_time += first_consonant.delay;
                off_time = off_time.max(first_consonant.transition_time);
            }
            else {
                off_time = off_time.min(first_consonant.transition_time);
            }
            for c in consonants.iter() {
                let mut time_scale = if legato {0.8} else {1.0};
                if consonants.len() > 1 {
                    time_scale *= 0.8;
                }
                // Emphasize the final consonant at the end of a line.
                let amplify = if legato || consonants.len() > 1 {1.0} else {1.2};
                let (delay_to_consonant, _delay_to_vowel, _envelope_offset) = self.add_consonant(delay, *c, final_vowel, true, note_index, time_scale, amplify);
                delay += delay_to_consonant;
            }
        }

        // Smoothly stop the sound.

        let end_envelope = if sustain {0.1} else {0.0};
        self.add_transition(stop_envelope_time, off_time, TransitionData::EnvelopeChange {start_envelope: self.envelope_after_transitions, end_envelope: end_envelope});
        self.current_note = None;
    }

    /// Add the Transitions to play a transient vowel (an initial or final vowel that sounds
    /// only briefly).
    fn add_transient_vowel(&mut self, delay: i64, envelope_offset: i64, prev_vowel: Option<char>, c: char, vowel_delay: i64, vowel_transition_time: i64, is_final: bool, legato: bool, note_index: i32) -> i64 {
        if prev_vowel.is_some() {
            self.add_vowel_transition(delay, prev_vowel.unwrap(), c, vowel_transition_time, note_index);
        }
        else {
            let shape = self.phonemes.get_vowel_shape(c).unwrap();
            let nasal_coupling = self.phonemes.get_nasal_coupling(c);
            self.add_shape_transition(delay, vowel_transition_time, shape.clone(), nasal_coupling, note_index, true);
        }
        let scale = if legato {0.9} else if is_final {0.25} else {0.7};
        let amplification = scale*self.phonemes.get_amplification(c);
        self.add_transition(delay-envelope_offset, vowel_transition_time, TransitionData::EnvelopeChange {
            start_envelope: self.envelope_after_transitions,
            end_envelope: amplification
        });
        delay+vowel_delay+vowel_transition_time
    }

    /// Add the Transitions to smoothly change the vocal tract shape between two vowels.
    fn add_vowel_transition(&mut self, delay: i64, vowel1: char, vowel2: char, vowel_transition_time: i64, note_index: i32) {
        let shape = self.phonemes.get_vowel_shape(vowel2).unwrap().clone();
        let nasal_coupling = self.phonemes.get_nasal_coupling(vowel2);
        if let Some(intermediate_shape) = self.phonemes.get_intermediate_shape(vowel1, vowel2) {
            let intermediate_coupling = 0.5*(self.nasal_coupling_after_transitions+nasal_coupling);
            self.add_shape_transition(delay, vowel_transition_time/2, intermediate_shape.clone(), intermediate_coupling, note_index, true);
            self.add_shape_transition(delay+vowel_transition_time/2, vowel_transition_time/2, shape, nasal_coupling, note_index, true);
        }
        else {
            self.add_shape_transition(delay, vowel_transition_time, shape, nasal_coupling, note_index, true);
        }
    }

    /// Play a consonant.  This adds a Consonant to the queue, and if necessary also adds a
    /// Transition to control the vocal tract shape appropriately.
    fn add_consonant(&mut self, delay: i64, c: char, adjacent_vowel: Option<char>, is_final: bool, note_index: i32, time_scale: f32, amplify: f32) -> (i64, i64, i64) {
        let mut consonant = self.phonemes.get_consonant(c, adjacent_vowel, is_final, time_scale).unwrap();
        consonant.start = self.step+delay+consonant.delay;
        consonant.volume *= 2.0*self.consonant_volume*amplify;
        let delay_to_consonant = consonant.delay+consonant.on_time+consonant.off_time;
        let mut delay_to_vowel = consonant.delay;
        let mut envelope_offset = 0;
        if !consonant.mono {
            consonant.volume *= (1.0+(self.max_voice_delay as f32 / 4000.0))/(self.voices.len() as f32).sqrt();
        }
        if consonant.samples.len() > 0 {
            // Some of the voices will use sampled consonants.  Randomly select the samples.

            let count = usize::max(1, self.voices.len()*3/4);
            consonant.sample_indices = self.random.get_indices(count, consonant.samples.len());
        }
        if let Some(vowel) = adjacent_vowel {
            let nasal_coupling = self.phonemes.get_nasal_coupling(vowel);
            if is_final {
                let end_shape = self.phonemes.get_consonant_shape(&consonant, vowel).unwrap().clone();
                self.add_shape_transition(delay, consonant.transition_time, end_shape, nasal_coupling, note_index, false);
                delay_to_vowel += consonant.transition_time;
            }
            else {
                let start_shape = self.phonemes.get_consonant_shape(&consonant, vowel).unwrap().clone();
                let end_shape = self.phonemes.get_vowel_shape(vowel).unwrap().clone();
                self.add_shape_transition(delay, 1000, start_shape, 0.0, note_index, false);
                self.add_shape_transition(delay+1000, consonant.transition_time, end_shape, nasal_coupling, note_index, true);
                delay_to_vowel += consonant.transition_time+1000;
            }
            if consonant.voiced {
                envelope_offset = consonant.transition_time;
            }
        }
        self.consonants.push(consonant);
        (delay_to_consonant, delay_to_vowel, envelope_offset)
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
    fn add_shape_transition(&mut self, delay: i64, duration: i64, mut end_shape: Vec<f32>, end_nasal_coupling: f32, note_index: i32, adjust_for_pitch: bool) {
        if end_nasal_coupling == 0.0 && self.brightness < 1.0 {
            let blend = (1.0-self.brightness)*0.2;
            for i in 0..end_shape.len() {
                end_shape[i] = (1.0-blend)*end_shape[i] + blend*self.dark_shape[i];
            }
        }
        if note_index > self.high_blend_note && end_nasal_coupling == 0.0 && adjust_for_pitch {
            let blend = self.high_blend_fraction * (note_index-self.high_blend_note) as f32 / (self.highest_note-self.high_blend_note) as f32;
            for i in 0..end_shape.len() {
                end_shape[i] = (1.0-blend)*end_shape[i] + blend*self.high_shape[i];
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
        if self.consonants.len() != 0 {
            self.off_after_step = self.step+500;
        }
        for e in &self.envelope {
            if *e > 0.0 {
                self.off_after_step = self.step+500;
            }
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
                for (k, consonant) in self.consonants.iter_mut().enumerate() {
                    // Mono consonants are only sung by one voice (the one panned to the center).
                    // Others are sung by every voice.

                    if !consonant.mono || i == self.voices.len()/2 {
                        let j = self.step-consonant.start-self.voice_delays[i];
                        let mut consonant_duration = consonant.on_time+consonant.off_time;
                        if j > 0 {
                            consonant_position = consonant.position;
                            if i < consonant.sample_indices.len() {
                                // Use a sampled consonant.

                                let index = consonant.sample_indices[i];
                                consonant_duration = consonant.samples[index].len() as i64;
                                if j < consonant_duration {
                                    consonant_noise = 50.0*consonant.volume*(consonant.samples[index][j as usize] as f32)/32768.0;
                                }
                            }
                            else {
                                // Use a synthesized consonant.

                                if j < consonant.on_time {
                                    let volume = consonant.volume*(j as f32 / consonant.on_time as f32);
                                    consonant_noise = volume*consonant.filter.process(2.0*self.random.get_uniform()-1.0);
                                }
                                else if j < consonant_duration {
                                    let k = j-consonant.on_time;
                                    let volume = consonant.volume*((consonant.off_time-k) as f32 / consonant.off_time as f32);
                                    consonant_noise = volume*consonant.filter.process(2.0*self.random.get_uniform()-1.0);
                                }
                            }
                        }
                        if consonant.mono {
                            consonant_noise *= (self.voices.len() as f32).sqrt();
                        }
                        if j < consonant_duration && k == 0 {
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
                        Message::NoteOn {syllable, note_index, velocity, continue_syllable} => {
                            let _ = self.note_on(&syllable, note_index, velocity, continue_syllable);
                        }
                        Message::NoteOff => {
                            self.note_off(false, false);
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
                        Message::SetReleaseRate {release} => {
                            self.release_rate = release;
                        }
                        Message::SetAccent {accent} => {
                            self.accent = accent;
                        }
                        Message::SetStereoWidth {width} => {
                            self.stereo_width = width;
                            self.update_pan_positions();
                        }
                        Message::SetMinVowelStartTime {samples} => {
                            self.min_vowel_start = samples;
                        }
                        Message::SetMaxVoiceDelay {max_delay} => {
                            self.max_voice_delay = max_delay;
                            self.update_voice_delays();
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
        let mut volume_changed = false;
        let mut frequency_changed = false;
        for transition in &self.transitions {
            for i in 0..self.voices.len() {
                let j = self.step-self.voice_delays[i];
                if j >= transition.start {
                    let fraction = (j-transition.start) as f32 / (transition.end-transition.start) as f32;
                    let weight2 = if j < transition.end {0.5-0.5*(fraction*std::f32::consts::PI).cos()} else {1.0};
                    let weight1 = 1.0-weight2;
                    match &transition.data {
                        TransitionData::EnvelopeChange {start_envelope, end_envelope} => {
                            self.envelope[i] = weight1*start_envelope + weight2*end_envelope;
                            volume_changed = true;
                        }
                        TransitionData::ShapeChange {start_shape, end_shape, start_nasal_coupling, end_nasal_coupling} => {
                            let coupling = weight1*start_nasal_coupling + weight2*end_nasal_coupling;
                            let n = start_shape[i].len();
                            let mut shape = vec![0.0; n];
                            for j in 0..n {
                                shape[j] = weight1*start_shape[i][j] + weight2*end_shape[i][j];
                            }
                            self.voices[i].set_vocal_shape(&shape, coupling);
                        }
                        TransitionData::FrequencyChange {start_frequency, end_frequency} => {
                            self.frequency[i] = weight1*start_frequency + weight2*end_frequency;
                            frequency_changed = true;
                        }
                    }
                }
            }
        }
        if volume_changed {
            self.update_volume();
        }
        if frequency_changed {
            self.update_frequency();
        }
        self.transitions.retain(|t| self.step < t.end+self.max_voice_delay);
    }

    /// Update the volumes of all Voices.  This is called whenever the Director's volume or
    /// envelope is changed.
    fn update_volume(&mut self) {
        let actual_volume = 0.05+0.95*self.volume;
        for i in 0..self.voices.len() {
            self.voices[i].set_volume(actual_volume*self.envelope[i]);
        }
    }

    /// Update the frequencies of all Voices.  This is called whenever the Director's frequency or
    /// pitch bend is changed.
    fn update_frequency(&mut self) {
        for i in 0..self.voices.len() {
            self.voices[i].set_frequency(self.frequency[i]*self.bend);
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
        let tremolo = 0.2*self.intensity;
        for voice in &mut self.voices {
            voice.set_noise(noise);
            voice.set_tremolo_amplitude(tremolo);
        }
        if let Some(note) = &self.current_note {
            let x = (self.highest_note-note.note_index) as f32 / (self.highest_note-self.lowest_note) as f32;
            let rd = 1.5 + 0.5*x - 0.2*self.volume - (self.intensity-0.5);
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

    /// Update the delay for each voice.
    fn update_voice_delays(&mut self) {
        let voice_count = self.voices.len();
        if voice_count == 1 {
            self.voice_delays[0] = 0;
        }
        else {
            for i in 0..voice_count {
                let index = ((i+(voice_count/2)) % voice_count) as i64;
                self.voice_delays[i] = self.max_voice_delay*index/(voice_count-1) as i64;
            }
        }
    }

    /// Get the timing parameters (delay, transition time) for a transient vowel.
    fn get_vowel_timing(&self, vowel: char, is_final: bool) -> (i64, i64) {
        if vowel == 'm' {
            if is_final {
                return (500, 3200);
            }
            return (0, 3200);
        }
        if vowel == 'n' {
            if is_final {
                return (500, 3200);
            }
            return (0, self.vowel_transition_time);
        }
        if vowel == 'N' {
            return (1000, 2000);
        }
        if vowel == 'l' && !is_final {
            return (0, 2000);
        }
        if vowel == '3' {
            if is_final {
                return (self.vowel_delay, 1500)
            }
            return (self.vowel_delay, 2500)
        }
        if is_final {
            return (self.vowel_delay, 3200)
        }
        (self.vowel_delay, self.vowel_transition_time)
    }
}