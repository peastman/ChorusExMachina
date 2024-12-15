pub mod voice;
pub mod phonemes;
pub mod director;
pub mod syllable;
mod random;
mod filter;

#[derive(Copy, Clone)]
pub enum VoicePart {
    Soprano,
    Alto,
    Tenor,
    Bass
}