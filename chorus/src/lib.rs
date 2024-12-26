pub mod voice;
pub mod phonemes;
pub mod director;
pub mod syllable;
pub mod random;
pub mod filter;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoicePart {
    Soprano,
    Alto,
    Tenor,
    Bass
}