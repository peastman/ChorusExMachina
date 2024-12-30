mod editor;

use chorus;
use chorus::director::{Director, Message};
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use std::sync::{Arc, Mutex, mpsc};

pub struct ChorusExMachina {
    params: Arc<ChorusExMachinaParams>,
    director: Arc<Mutex<Director>>,
    sender: Arc<Mutex<mpsc::Sender<Message>>>,
    last_note: u8,
    last_dynamics: f32,
    last_vibrato: f32,
    last_stereo_width: f32
}

#[derive(Params)]
struct ChorusExMachinaParams {
    #[persist = "editor_state"]
    editor_state: Arc<EguiState>,
    #[id = "voice_part"]
    pub voice_part: EnumParam<VoicePart>,
    #[id = "voice_count"]
    pub voice_count: IntParam,
    #[id = "dynamics"]
    pub dynamics: FloatParam,
    #[id = "vibrato"]
    pub vibrato: FloatParam,
    #[id = "stereo_width"]
    pub stereo_width: FloatParam,
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
            last_note: 255,
            last_dynamics: -1.0,
            last_vibrato: -1.0,
            last_stereo_width: -1.0
        }
    }
}

impl Default for ChorusExMachinaParams {
    fn default() -> Self {
        Self {
            editor_state: EguiState::from_size(600, 400),
            voice_part: EnumParam::new("Voice Part", VoicePart::Soprano).non_automatable(),
            voice_count: IntParam::new("Voices", 4, IntRange::Linear {min: 1, max: 8}).non_automatable(),
            dynamics: FloatParam::new("Dynamics", 1.0, FloatRange::Linear {min: 0.0, max: 1.0}),
            vibrato: FloatParam::new("Vibrato", 0.4, FloatRange::Linear {min: 0.0, max: 1.0}),
            stereo_width: FloatParam::new("Stereo Width", 0.3, FloatRange::Linear {min: 0.0, max: 1.0})
        }
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

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, _buffer_config: &BufferConfig, _context: &mut impl InitContext<Self>) -> bool {
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
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
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
        if self.last_stereo_width != self.params.stereo_width.value() {
            self.last_stereo_width = self.params.stereo_width.value();
            let _ = sender.send(Message::SetStereoWidth {width: self.last_stereo_width});
        }
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() != sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        let _ = sender.send(Message::NoteOn {syllable: "A".to_string(), note_index: note as i32, velocity: velocity});
                        self.last_note = note;
                    },
                    NoteEvent::NoteOff { note, .. } => {
                        if note == self.last_note {
                            let _ = sender.send(Message::NoteOff);
                        }
                    },
                    NoteEvent::MidiPitchBend { value, .. } => {
                        let _ = sender.send(Message::SetPitchBend {semitones: 4.0*(value-0.5)});
                    },
                    _ => (),
                }
                next_event = context.next_event();
            }
            let (left, right) = director.generate();
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
        let params = self.params.clone();
        let sender = Arc::clone(&self.sender);
        editor::draw_editor(params, sender)
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
