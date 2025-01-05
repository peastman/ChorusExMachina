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

pub mod voice;
pub mod phonemes;
pub mod director;
pub mod syllable;
pub mod random;
pub mod resampler;
pub mod filter;

pub const SAMPLE_RATE: i32 = 48000;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum VoicePart {
    Soprano,
    Alto,
    Tenor,
    Bass
}