use std::mem::replace;

use crate::{
    editing::{
        chart_path::{ChartPath, LinePath},
        ChartConflictError,
    },
    prelude::*,
};

use super::ChartCommand;
pub struct InsertLine {
    pub line: Line,
    pub at: Option<usize>,
}

impl ChartCommand for InsertLine {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let Self { line, at } = self;
        let len = chart.lines.len();
        let at_clamped = at.unwrap_or(len).clamp(0, len);
        chart.lines.insert(at_clamped, line);
        Ok(RemoveLine {
            line_path: at_clamped.into(),
        }
        .into())
    }
    fn validate(&self,_chart: &Chart) -> crate::editing::Result<()> {
        Ok(())
    }
}

pub struct RemoveLine {
    pub line_path: LinePath,
}

impl ChartCommand for RemoveLine {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let line = self.line_path.remove(chart)?;
        Ok(InsertLine {
            line,
            at: Some(self.line_path.0),
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> crate::editing::Result<()> {
        self.line_path.valid(chart)
    }
}

pub struct EditPoint {
    pub line_path: LinePath,
    pub point_idx: usize,
    pub new_time: Option<f32>,
    pub new_x: Option<f32>,
    pub new_canvas: Option<usize>,
    pub new_color: Option<ColorRGBA>,
    pub new_easing: Option<EasingId>,
}

impl ChartCommand for EditPoint {
    fn apply(mut self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let len = chart.canvases.len();
        let line = self.line_path.get_mut(chart)?;
        let prev_time = line
            .points
            .points
            .get(self.point_idx - 1)
            .map(|point| point.time)
            .unwrap_or(f32::NEG_INFINITY);
        let next_time = line
            .points
            .points
            .get(self.point_idx + 1)
            .map(|point| point.time)
            .unwrap_or(f32::INFINITY);
        let point =
            line.points
                .points
                .get_mut(self.point_idx)
                .ok_or(ChartConflictError::NoSuchPoint {
                    line_path: self.line_path,
                    point: self.point_idx,
                })?;
        let mut old_canvas = None;
        if let Some(canvas) = self.new_canvas {
            if canvas < len {
                old_canvas = Some(point.relevant.canvas);
                point.relevant.canvas = canvas;
            } else {
                return Err(ChartConflictError::NoSuchCanvas { canvas });
            }
        }

        self.new_time = self.new_time.map(|new| new.clamp(prev_time, next_time));
        let old_color = self
            .new_color
            .map(|color| replace(&mut point.relevant.color, color));
        Ok(Self {
            line_path: self.line_path,
            point_idx: self.point_idx,
            new_time: self.new_time.map(|new| replace(&mut point.time, new)),
            new_x: self.new_x.map(|new| replace(&mut point.value, new)),
            new_canvas: old_canvas,
            new_color: old_color,
            new_easing: self.new_easing.map(|new| replace(&mut point.ease_type, new))
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> crate::editing::Result<()> {
        let canvas_len = chart.canvases.len();
        if let Some(canvas) = self.new_canvas{
            if canvas >= canvas_len {
                return Err(ChartConflictError::NoSuchCanvas { canvas });
            }
        }
        let line = self.line_path.get(chart)?;
        if self.point_idx >= line.points.len() {
            Err(ChartConflictError::NoSuchPoint {
                line_path: self.line_path,
                point: self.point_idx,
            })
        }
        else {
            Ok(())
        }

    }
}

pub struct InsertPoint {
    pub line_path: LinePath,
    pub point_idx: Option<usize>,
    pub point: KeyPoint<f32, LinePointData>,
}

impl ChartCommand for InsertPoint {
    fn apply(mut self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let canvas_len = chart.canvases.len();
        if self.point.relevant.canvas >= canvas_len {
            return Err(ChartConflictError::NoSuchCanvas {
                canvas: self.point.relevant.canvas,
            });
        }

        let line = self.line_path.get_mut(chart)?;
        let at = self
            .point_idx
            .unwrap_or(line.points.len())
            .clamp(0, line.points.len());
        let prev_time = line
            .points
            .points
            .get(at - 1)
            .map(|point| point.time)
            .unwrap_or(f32::NEG_INFINITY);
        let next_time = line
            .points
            .points
            .get(at + 1)
            .map(|point| point.time)
            .unwrap_or(f32::INFINITY);
        self.point.time = self.point.time.clamp(prev_time, next_time);
        line.points.points.insert(at, self.point);
        Ok(RemovePoint {
            line_path: self.line_path,
            point_idx: at,
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> crate::editing::Result<()> {
        self.line_path.valid(chart)
    }
}

pub struct RemovePoint {
    pub line_path: LinePath,
    pub point_idx: usize,
}

impl ChartCommand for RemovePoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let point = self
            .line_path
            .get_mut(chart)?
            .points
            .remove(self.point_idx)
            .ok_or(ChartConflictError::NoSuchPoint {
                line_path: self.line_path,
                point: self.point_idx,
            })?;
        Ok(InsertPoint {
            line_path: self.line_path,
            point_idx: Some(self.point_idx),
            point,
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> crate::editing::Result<()> {
        let points_len = self.line_path.get(chart)?.points.len();
        if points_len < self.point_idx {
            Err(ChartConflictError::NoSuchPoint {
                line_path: self.line_path,
                point: self.point_idx,
            })
        }
        else {
            Ok(())
        }
    }
}
