#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub enum NoteKind {
    Tap,
    Hold { end: f32 },
    Drag,
}

/// 单个的Note.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct Note {
    pub time: f32,
    pub kind: NoteKind,
}

impl Note {
    pub const fn new(time: f32, kind: NoteKind) -> Self {
        Self { time, kind }
    }
}
