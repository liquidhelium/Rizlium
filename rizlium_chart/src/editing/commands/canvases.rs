use std::borrow::Cow;

use crate::editing::chart_path::{CanvasPath, ChartPath as _};
use crate::prelude::Chart;
#[derive(Debug)]
pub struct RemoveCanvas {
    pub canvas_path: CanvasPath,
}
impl super::ChartCommand for RemoveCanvas {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let removed = self.canvas_path.remove(chart)?;
        Ok(InsertCanvas { canvas: removed, at: Some(self.canvas_path.0) }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        self.canvas_path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Remove Canvas {}", self.canvas_path.0).into()
    }
}
#[derive(Debug)]
pub struct InsertCanvas {
    pub canvas: crate::prelude::Canvas,
    pub at: Option<usize>,
}
impl super::ChartCommand for InsertCanvas {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let len = chart.canvases.len();
        if let Some(at) = self.at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds {
                    index: at,
                    len,
                });
            }
        }
        let at_clamped = self.at.unwrap_or(len);
        chart.canvases.insert(at_clamped, self.canvas);
        Ok(RemoveCanvas {
            canvas_path: at_clamped.into(),
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        let len = chart.canvases.len();
        if let Some(at) = self.at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds {
                    index: at,
                    len,
                });
            }
        }
        Ok(())
    }
    fn description(&self) -> Cow<'static, str> {
        format!(
            "Insert Canvas at {}",
            self.at
                .map(|at| at.to_string())
                .unwrap_or_else(|| "end".to_string())
        ).into()
    }
}
