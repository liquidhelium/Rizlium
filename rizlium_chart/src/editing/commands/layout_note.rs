use std::borrow::Cow;

use crate::editing::chart_path::{ChartPath, LayoutNotePath};
use crate::prelude::{Chart, LayoutNote};

use crate::editing::{
    commands::{ChartCommand, ChartCommands},
    Result,
};

/// 修改 LayoutNote 的时间
#[derive(Debug, Clone)]
pub struct ChangeLayoutNoteTime {
    pub modify_to: f32,
    pub note_path: LayoutNotePath,
}

impl ChartCommand for ChangeLayoutNoteTime {
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
    fn validate(&self, chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        "Change layout note time".into()
    }
}

/// 修改 LayoutNote 的 x 位置
#[derive(Debug, Clone)]
pub struct ChangeLayoutNoteX {
    pub modify_to: f32,
    pub note_path: LayoutNotePath,
}

impl ChartCommand for ChangeLayoutNoteX {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let note = self.note_path.get_mut(chart)?;
        let current_x = note.x;
        note.x = self.modify_to;
        Ok(Self {
            modify_to: current_x,
            note_path: self.note_path,
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        "Change layout note x position".into()
    }
}

/// 同时修改 LayoutNote 的时间和 x 位置
#[derive(Debug, Clone)]
pub struct MoveLayoutNote {
    pub new_time: f32,
    pub new_x: f32,
    pub note_path: LayoutNotePath,
}

impl ChartCommand for MoveLayoutNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let note = self.note_path.get_mut(chart)?;
        let old_time = note.time;
        let old_x = note.x;
        note.time = self.new_time;
        note.x = self.new_x;
        Ok(Self {
            new_time: old_time,
            new_x: old_x,
            note_path: self.note_path,
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        "Move layout note".into()
    }
}

/// 插入 LayoutNote
#[derive(Debug, Clone)]
pub struct InsertLayoutNote {
    pub note: LayoutNote,
    pub at: Option<usize>,
}

impl ChartCommand for InsertLayoutNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self { note, at } = self;
        let at_clamped = at.unwrap_or(chart.layout_notes.len()).clamp(0, chart.layout_notes.len());
        chart.layout_notes.insert(at_clamped, note);
        Ok(RemoveLayoutNote {
            note_path: LayoutNotePath(at_clamped),
        }
        .into())
    }
    fn validate(&self, _chart: &Chart) -> Result<()> {
        Ok(())
    }
    fn description(&self) -> Cow<'static, str> {
        "Insert layout note".into()
    }
}

/// 移除 LayoutNote
#[derive(Debug, Clone)]
pub struct RemoveLayoutNote {
    pub note_path: LayoutNotePath,
}

impl ChartCommand for RemoveLayoutNote {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let Self {
            note_path: LayoutNotePath(note_idx),
        } = self;
        let note = self.note_path.remove(chart)?;

        Ok(InsertLayoutNote {
            note,
            at: Some(note_idx),
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        "Remove layout note".into()
    }
}

/// 修改 LayoutNote 的类型
#[derive(Debug, Clone)]
pub struct ChangeLayoutNoteKind {
    pub new_kind: crate::prelude::NoteKind,
    pub note_path: LayoutNotePath,
}

impl ChartCommand for ChangeLayoutNoteKind {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let note = self.note_path.get_mut(chart)?;
        let old_kind = note.kind.clone();
        note.kind = self.new_kind;
        Ok(Self {
            new_kind: old_kind,
            note_path: self.note_path,
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> Result<()> {
        self.note_path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        "Change layout note kind".into()
    }
}
