use std::borrow::Cow;

use crate::prelude::Chart;
use snafu::Snafu;

use self::chart_path::LinePath;
pub use self::{
    chart_path::NotePath,
    commands::{ChartCommand, ChartCommands},
};
/// Representation of a chart item
pub mod chart_path;
pub mod commands;

#[derive(Snafu, Debug)]
pub enum ChartConflictError {
    #[snafu(display("Invalid note path: {note_path:?}"))]
    InvalidNotePath {
        note_path: NotePath,
    },
    #[snafu(display("Invalid line path: {line_path:?}"))]
    InvalidLinePath {
        line_path: LinePath,
    },
    #[snafu(display("No such point {point} in line {line_path:?}"))]
    NoSuchPoint {
        line_path: LinePath,
        point: usize,
    },
    #[snafu(display("Time {time} out of bound for point {point} in line {line_path:?}"))]
    TimeOutBound {
        line_path: LinePath,
        point: usize,
        time: f32,
    },
    #[snafu(display("No such canvas {canvas}"))]
    NoSuchCanvas {
        canvas: usize,
    },
    #[snafu(display("No such global spline point {index} in spline {spline}"))]
    NoSuchGlobalSplinePoint {
        spline: &'static str,
        index: usize,
    },
    #[snafu(display("Index {index} out of bounds (len: {len})"))]
    IndexOutOfBounds {
        index: usize,
        len: usize,
    },
    #[snafu(display("Preediting in progress"))]
    PreeditingInProgress,
}


type Result<T> = std::result::Result<T, ChartConflictError>;

#[derive(Default)]
pub struct EditHistory {
    history_descriptions: Vec<Cow<'static, str>>,
    inverse_history: Vec<ChartCommands>,
    preedit_data: Vec<PreeditData>,
    redo_cache: Vec<ChartCommands>,
}

pub struct PreeditData {
    inverse: ChartCommands,
    description: Cow<'static, str>,
}

impl PreeditData {
    pub fn inverse(&self) -> &ChartCommands {
        &self.inverse
    }

    pub fn description(&self) -> &str {
        &self.description
    }
}

impl EditHistory {
    pub fn push(&mut self, edit: impl Into<ChartCommands>, chart: &mut Chart) -> Result<()> {
        if self.has_preedit() {
            return Err(ChartConflictError::PreeditingInProgress);
        }
        let command = edit.into();
        self.push_direct(command, chart)?;
        self.redo_cache.clear();
        Ok(())
    }

    fn push_direct(&mut self, command: ChartCommands, chart: &mut Chart) -> Result<()> {
        command.validate(chart)?;
        let desc = command.description();
        let inversed = command.apply(chart)?;
        self.inverse_history.push(inversed);
        self.history_descriptions.push(desc);
        Ok(())
    }
    pub fn replace_last_preedit(
        &mut self,
        edit: impl Into<ChartCommands>,
        chart: &mut Chart,
    ) -> Result<()> {
        let command: ChartCommands = edit.into();
        command.validate(chart)?;
        self.discard_last_preedit(chart)?;
        let desc = command.description();
        let command_inversed = command.apply(chart)?;
        self.preedit_data.push(PreeditData {
            inverse: command_inversed,
            description: desc,
        });
        Ok(())
    }

    pub fn discard_last_preedit(&mut self, chart: &mut Chart) -> Result<()> {
        if let Some(last) = self.preedit_data.pop() {
            last.inverse.apply(chart)?;
        };
        Ok(())
    }
    pub fn push_preedit(
        &mut self,
        edit: impl Into<ChartCommands>,
        chart: &mut Chart,
    ) -> Result<()> {
        let command: ChartCommands = edit.into();
        command.validate(chart)?;
        let desc = command.description();
        let command_inversed = command.apply(chart)?;
        self.preedit_data.push(PreeditData {
            inverse: command_inversed,
            description: desc,
        });
        Ok(())
    }
    pub fn discard_preedit(&mut self, chart: &mut Chart) -> Result<()> {
        self.preedit_data.drain(..).rev().try_for_each(|data| {
            data.inverse.apply(chart)?;
            Ok(())
        })?;
        Ok(())
    }
    pub fn submit_preedit(&mut self) {
        let (mut v1, mut v2): (Vec<_>, Vec<_>) = self
            .preedit_data
            .drain(..)
            .map(|data| (data.inverse, data.description))
            .unzip();
        self.inverse_history.append(&mut v1);
        self.history_descriptions.append(&mut v2);
        self.redo_cache.clear();
    }
    /// Like `submit_preedit`, but also squash preedits into a single command.
    pub fn submit_preedit_squash(&mut self, description: impl Into<Cow<'static, str>>) {
        let v1: Vec<_> = self
            .preedit_data
            .drain(..)
            .map(|data| data.inverse)
            // reverse to ensure inversed commands get processed in the correct order
            // logic here is similar to CommandSequence::apply
            .rev()
            .collect();
        let squashed_command = commands::CommandSequence {
            commands: v1,
            description: description.into(),
        };
        let desc = squashed_command.description();
        self.inverse_history.push(squashed_command.into());
        self.history_descriptions.push(desc);
        self.redo_cache.clear();
    }
    pub fn history_descriptions(&self) -> &[Cow<'static, str>] {
        self.history_descriptions.as_slice()
    }

    pub fn preedit_datas(&self) -> &[PreeditData] {
        self.preedit_data.as_slice()
    }

    pub fn inverse(&self) -> &[ChartCommands] {
        self.inverse_history.as_slice()
    }

    pub fn redo_inverse(&self) -> &[ChartCommands] {
        self.redo_cache.as_slice()
    }

    pub fn gen_redo_descriptions(&self) -> impl ExactSizeIterator<Item = Cow<'static, str>> + '_ {
        self.redo_cache.iter().map(|c| c.description())
    }

    pub fn undo(&mut self, chart: &mut Chart) -> Result<()> {
        self.discard_preedit(chart)?;
        let Some(history) = self.inverse_history.pop() else {
            return Ok(());
        };
        self.history_descriptions.pop();
        let inversed = history.apply(chart)?;
        self.redo_cache.push(inversed);
        Ok(())
    }
    pub fn redo(&mut self, chart: &mut Chart) -> Result<()> {
        self.discard_preedit(chart)?;
        let Some(redo) = self.redo_cache.pop() else {
            return Ok(());
        };
        self.push_direct(redo, chart)
    }

    pub fn can_undo(&self) -> bool {
        !self.inverse_history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_cache.is_empty()
    }

    pub fn has_preedit(&self) -> bool {
        !self.preedit_data.is_empty()
    }
}
