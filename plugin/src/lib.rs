use chorus;
use chorus::director::{Director, Message};
use nih_plug::prelude::*;
use std::sync::{Arc, mpsc};

pub struct ChorusExMachina {
    params: Arc<ChorusExMachinaParams>,
    director: Option<Director>,
    sender: Option<mpsc::Sender<Message>>,
    last_note: u8
}

#[derive(Params)]
struct ChorusExMachinaParams {
    #[id = "voice_part"]
    pub voice_part: EnumParam<VoicePart>,
    #[id = "voices"]
    pub voices: IntParam,
}

#[derive(Enum, Debug, PartialEq)]
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
        Self {
            params: Arc::new(ChorusExMachinaParams::default()),
            director: None,
            sender: None,
            last_note:255
        }
    }
}

impl Default for ChorusExMachinaParams {
    fn default() -> Self {
        Self {
            voice_part: EnumParam::new("Voice Part", VoicePart::Soprano).non_automatable(),
            voices: IntParam::new("Voices", 4, IntRange::Linear {min:1, max:8}).non_automatable()
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
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(&mut self, _audio_io_layout: &AudioIOLayout, _buffer_config: &BufferConfig, context: &mut impl InitContext<Self>) -> bool {
        let voice_part = match self.params.voice_part.value() {
            VoicePart::Soprano => chorus::VoicePart::Soprano,
            VoicePart::Alto => chorus::VoicePart::Alto,
            VoicePart::Tenor => chorus::VoicePart::Tenor,
            VoicePart::Bass => chorus::VoicePart::Bass,
        };
        let voices = self.params.voices.value() as usize;
        let (sender, receiver) = mpsc::channel();
        self.director = Some(Director::new(voice_part, voices, receiver));
        self.sender = Some(sender);
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }

    fn process(&mut self, buffer: &mut Buffer, _aux: &mut AuxiliaryBuffers, context: &mut impl ProcessContext<Self>) -> ProcessStatus {
        if self.director.is_none() || self.sender.is_none() {
            return ProcessStatus::KeepAlive;
        }
        let director = self.director.as_mut().unwrap();
        let sender = &self.sender.as_mut().unwrap();
        let mut next_event = context.next_event();
        for (sample_id, channel_samples) in buffer.iter_samples().enumerate() {
            while let Some(event) = next_event {
                if event.timing() != sample_id as u32 {
                    break;
                }
                match event {
                    NoteEvent::NoteOn { note, velocity, .. } => {
                        let _ = sender.send(Message::NoteOn {syllable: "A".to_string(), note_index: note as i32, velocity: velocity} );
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
