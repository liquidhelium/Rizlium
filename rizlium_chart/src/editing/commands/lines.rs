use crate::{editing::chart_path::{ChartPath, LinePath}, prelude::*};

use super::ChartCommand;
pub struct InsertLine {
    pub line: Line,
    pub at: Option<usize>,
}

impl ChartCommand for InsertLine {
    fn apply(self,chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let Self {
            line,
            at
        } = self;
        let len = chart.lines.len();
        let at_clamped = at.unwrap_or(len).clamp(0, len);
        chart.lines.insert(at_clamped, line);
        Ok(RemoveLine {
            line_path: at_clamped.into()
        }.into())
    }
}

pub struct RemoveLine {
    pub line_path: LinePath,
}

impl ChartCommand for RemoveLine {
    fn apply(self,chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let line = self.line_path.remove(chart)?;
        Ok(InsertLine {
            line,
            at: Some(self.line_path.0)
        }.into())
    }
}