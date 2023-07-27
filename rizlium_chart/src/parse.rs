use snafu::prelude::*;

pub mod official;

#[derive(Debug, Snafu,Clone)]
pub enum ConvertError {
    #[snafu(display("No bpm data found"))]
    EmptyBPM,
    #[snafu(display("Hold at line {line_idx}, index {note_idx} has no end"))]
    HoldNoEnd { line_idx: usize, note_idx: usize },
    #[snafu(display("Unknown note kind: {raw_kind}"))]
    UnknownNoteKind { raw_kind: usize },
    #[snafu(display("Unknown ease kind: {raw_kind}"))]
    UnknownEaseKind { raw_kind: u8}
}

type ConvertResult<T, E = ConvertError> = std::result::Result<T, E>;
