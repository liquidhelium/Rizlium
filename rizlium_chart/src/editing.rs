use crate::prelude::Chart;
#[cfg(test)]
use crate::test_resources;
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
    InvalidNotePath { note_path: NotePath },
    InvalidLinePath { line_path: LinePath },
    NoSuchPoint {line_path: LinePath, point: usize},
    TimeOutBound {line_path: LinePath, point: usize, time: f32},
    NoSuchCanvas {canvas: usize},
}

type Result<T> = std::result::Result<T, ChartConflictError>;

#[derive(Default)]
pub struct EditHistory {
    _history_descriptions: Vec<String>,
    inverse_history: Vec<ChartCommands>,
    last_preedit_inverse: Option<ChartCommands>,
}

impl EditHistory {
    pub fn push(&mut self, edit: impl Into<ChartCommands>, chart: &mut Chart) -> Result<()> {
        // TODO: desc
        let inversed = edit.into().apply(chart)?;
        self.inverse_history.push(inversed);
        Ok(())
    }
    pub fn push_preedit(&mut self, edit: impl Into<ChartCommands>, chart: &mut Chart) -> Result<()> {
        self.discard_preedit(chart)?;
        let current_inverse = edit.into().apply(chart)?;
        self.last_preedit_inverse = Some(current_inverse);
        Ok(())
    }
    pub fn discard_preedit(&mut self, chart: &mut Chart) -> Result<()> {
        if let Some(last) = self.last_preedit_inverse.take() {
            last.apply(chart)?;
        }
        Ok(())
    }
    pub fn submit_preedit(&mut self) {
        if let Some(last) = self.last_preedit_inverse.take() {
            self.inverse_history.push(last)
        }
    }
}


