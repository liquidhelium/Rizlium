use std::borrow::Cow;

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
    InvalidNotePath {
        note_path: NotePath,
    },
    InvalidLinePath {
        line_path: LinePath,
    },
    NoSuchPoint {
        line_path: LinePath,
        point: usize,
    },
    TimeOutBound {
        line_path: LinePath,
        point: usize,
        time: f32,
    },
    NoSuchCanvas {
        canvas: usize,
    },
}

type Result<T> = std::result::Result<T, ChartConflictError>;

#[derive(Default)]
pub struct EditHistory {
    history_descriptions: Vec<Cow<'static, str>>,
    inverse_history: Vec<ChartCommands>,
    preedit_data: Vec<PreeditData>,
}

struct PreeditData {
    inverse: ChartCommands,
    description: Cow<'static, str>,
}

impl EditHistory {
    pub fn push(&mut self, edit: impl Into<ChartCommands>, chart: &mut Chart) -> Result<()> {
        let command = edit.into();
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
        if let Some(last) = self.preedit_data.pop() {
            last.inverse.apply(chart)?;
        }
        let desc = command.description();
        let command_inversed = command.apply(chart)?;
        self.preedit_data.push(PreeditData {
            inverse: command_inversed,
            description: desc,
        });
        Ok(())
    }
    pub fn push_preedit(&mut self, edit: impl Into<ChartCommands>, chart: &mut Chart) -> Result<()> {
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
        self.preedit_data.drain(..).try_for_each(|data| {
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
    }
    pub fn history_descriptions(&self) -> &[Cow<'static, str>] {
        self.history_descriptions.as_slice()
    }
}
