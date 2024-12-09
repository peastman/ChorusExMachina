use crate::synth::Voice;
use crate::phonemes::Phonemes;

pub struct Transition {
    start: i64,
    end: i64,
    data: TransitionData
}

pub enum TransitionData {
    VolumeChange {start_volume: f32, end_volume: f32},
    ShapeChange {start_shape: Vec<f32>, end_shape: Vec<f32>, start_nasal_coupling: f32, end_nasal_coupling: f32}
}

pub struct Director {
    voices: Vec<Voice>,
    step: i64,
    transitions: Vec<Transition>
}

impl Director {
    pub fn new(voices: Vec<Voice>) -> Self {
        Self {
            voices: voices,
            step: 0,
            transitions: vec![]
        }
    }

    pub fn generate(&mut self) -> f32 {
        if self.step%200 == 0 {
            self.update_transitions();
        }
        let mut sum = 0.0;
        for voice in &mut self.voices {
            sum += voice.generate(self.step);
        }
        self.step += 1;
        0.04*sum
    }

    pub fn add_transition(&mut self, delay: i64, duration: i64, data: TransitionData) {
        let transition = Transition { start: self.step+delay, end: self.step+delay+duration, data: data };
        self.transitions.push(transition);
    }

    fn update_transitions(&mut self) {
        for transition in &self.transitions {
            if self.step >= transition.start {
                let fraction = (self.step-transition.start) as f32 / (transition.end-transition.start) as f32;
                let weight2 = if self.step < transition.end {0.5-0.5*(fraction*std::f32::consts::PI).cos()} else {1.0};
                let weight1 = 1.0-weight2;
                match &transition.data {
                    TransitionData::VolumeChange {start_volume, end_volume} => {
                        for voice in &mut self.voices {
                            voice.set_volume(weight1*start_volume + weight2*end_volume);
                        }
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
                }
            }
        }
        self.transitions.retain(|t| self.step < t.end);
    }
}