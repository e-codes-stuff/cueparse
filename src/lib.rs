#![forbid(unsafe_code)]

use std::{path::PathBuf, time::Duration};

mod parser;

#[derive(Debug, Clone, Default)]
pub struct Cue {
    pub catalog: Option<String>,
    pub cd_text_file: Option<PathBuf>,
    pub path: Option<PathBuf>,
    pub format: FileFormat,
    pub performer: Option<String>,
    pub songwriter: Option<String>,
    pub arranger: Option<String>,
    pub title: Option<String>,
    pub tracks: Vec<Track>,
    pub comments: Vec<String>,
}

impl Cue {
    pub fn from_str(input: impl AsRef<str>) -> Result<Self, Error> {
        parser::parse_cue(input)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Track {
    pub track_index: u8,
    pub indices: Vec<TrackIndex>,
    pub time: Option<Frames>,
    pub flags: TrackFlags,
    pub mode: TrackMode,
    pub file: Option<PathBuf>,
    pub format: FileFormat,
    pub performer: Option<String>,
    pub songwriter: Option<String>,
    pub title: Option<String>,
    pub isrc: Option<String>,
    pub pregap: Option<Frames>,
    pub postgap: Option<Frames>,
    pub comments: Vec<String>,
    pub arranger: Option<String>,
}

impl Track {
    pub fn new(track_index: u8, mode: TrackMode) -> Self {
        Self {
            track_index,
            mode,
            ..Default::default()
        }
    }

    pub fn set_file(&mut self, path: impl Into<PathBuf>, format: FileFormat) {
        self.file = Some(path.into());
        self.format = format;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FileFormat {
    #[default]
    Unspecified,
    Binary,
    Motorola,
    Aiff,
    Wave,
    Mp3,
}

bitflags::bitflags! {
    #[derive(Default)]
    pub struct TrackFlags: u8 {
        const PRE_EMPHASIS_ENABLED          = 0b00000001;
        const DIGITAL_COPY_PERMITTED        = 0b00000010;
        const FOUR_CHANNEL                  = 0b00000100;
        const SERIAL_COPY_MANAGEMENT_SYSTEM = 0b00001000;
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TrackMode {
    #[default]
    Audio,
    Cdg,
    Mode1_2048,
    Mode1_2352,
    Mode2_2336,
    Mode2_2352,
    Cdi_2336,
    Cdi_2352,
}

#[derive(Debug, Clone)]
pub struct TrackIndex {
    index: usize,
    time: Option<Frames>,
}

/// [`Frames`] is a struct representing a count of 1/75th of a second frames used in CDs
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Frames(usize);

impl Frames {
    // 1 frame = 1/75th of a second
    const FRAME_LENGTH_F32: f32 = 1.0 / 75.0;
    const FRAME_LENGTH_F64: f64 = 1.0 / 75.0;

    pub fn new(frames: usize) -> Self {
        Self(frames)
    }

    /// From MM:SS:FF (Minutes/Seconds/Frames) format
    pub fn from_msf(m: usize, s: usize, f: usize) -> Self {
        let frames = ((m * 60) + s) * 75 + f;
        Self(frames)
    }

    fn to_msf(&self) -> (usize, usize, usize) {
        let mut frames = self.0;

        let f = frames % 75;

        frames /= 75;

        let s = frames % 60;

        frames /= 60;

        let m = frames;

        (m, s, f)
    }

    pub fn to_secs_f32(self) -> f32 {
        self.0 as f32 * Self::FRAME_LENGTH_F32
    }

    pub fn to_secs_f64(self) -> f64 {
        self.0 as f64 * Self::FRAME_LENGTH_F64
    }

    pub fn to_duration(self) -> Duration {
        Duration::from_secs_f64(self.to_secs_f64())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ParseError(#[from] pest::error::Error<parser::Rule>),
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_cue;

    use super::*;

    static CUE_EXAMPLE: &str = include_str!("../test_files/example.cue");

    #[test]
    fn parse_example() {
        let res = parse_cue(CUE_EXAMPLE);

        match res {
            Ok(ref cue) => println!("{:#?}", cue),
            Err(ref e) => println!("{e}"),
        }

        assert!(res.is_ok())
    }
}
