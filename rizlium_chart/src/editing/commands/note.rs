use crate::editing::chart_path::{ChartPath, LinePath};
use crate::prelude::{Chart, Note};

use crate::editing::{
    chart_path::NotePath,
    commands::{ChartCommand, ChartCommands},
    Result,
};

pub struct ChangeNoteTime {
    pub modify_to: f32,
    pub note_path: NotePath,
}

impl ChartCommand for ChangeNoteTime {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let note = self.note_path.get_mut(chart)?;
        let current_time = note.time;
        note.time = self.modify_to;
        Ok(Self {
            modify_to: current_time,
            note_path: self.note_path,
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
}

pub struct InsertNote {
    pub note: Note,
    pub line: LinePath,
    pub at: Option<usize>,
}

impl ChartCommand for InsertNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self { note, line, at } = self;
        let notes = &mut line.get_mut(chart)?.notes;
        let at_clamped = at.unwrap_or(notes.len()).clamp(0, notes.len());
        notes.insert(at_clamped, note);
        Ok(RemoveNote {
            note_path: NotePath(line, at_clamped),
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.line.valid(chart)
    }
}

pub struct RemoveNote {
    pub note_path: NotePath,
}

impl ChartCommand for RemoveNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self {
            note_path: NotePath(line_idx, note_idx),
        } = self;
        let note = self.note_path.remove(chart)?;

        Ok(InsertNote {
            note,
            line: line_idx,
            at: Some(note_idx),
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
}
// here used to have a test