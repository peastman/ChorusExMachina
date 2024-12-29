pub mod voice;
pub mod phonemes;
pub mod director;
pub mod syllable;
pub mod random;
pub mod filter;

pub const SAMPLE_RATE: i32 = 48000;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoicePart {
    Soprano,
    Alto,
    Tenor,
    Bass
}