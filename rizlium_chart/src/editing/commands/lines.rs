use std::mem::replace;

use crate::{
    editing::{
        chart_path::{ChartPath, LinePath, LineColorPath, RingColorPath},
        ChartConflictError,
    },
    prelude::*,
};

use super::ChartCommand;
#[derive(Debug, Clone)]
pub struct InsertLine {
    pub line: Line,
    pub at: Option<usize>,
}

impl ChartCommand for InsertLine {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let Self { line, at } = self;
        let len = chart.lines.len();

        if let Some(at) = at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds {
                    index: at,
                    len,
                });
            }
        }
        let at_clamped = at.unwrap_or(len);
        chart.lines.insert(at_clamped, line);
        Ok(RemoveLine {
            line_path: at_clamped.into(),
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        let len = chart.lines.len();
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
}

#[derive(Debug, Clone)]
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
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.line_path.valid(chart)
    }
}

#[derive(Debug, Clone)]
pub struct RenameLine {
    pub line_path: LinePath,
    pub name: String,
}

impl ChartCommand for RenameLine {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let line = self.line_path.get_mut(chart)?;
        let old_name = replace(&mut line.name, self.name.clone());
        Ok(RenameLine {
            line_path: self.line_path,
            name: old_name,
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.line_path.valid(chart)
    }
    fn description(&self) -> std::borrow::Cow<'static, str> {
        format!("Rename Line {} to {}", self.line_path.0, self.name).into()
    }
}

#[derive(Debug, Default, Clone)]
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
        let prev_time = if self.point_idx > 0 {
            line.points
                .points
                .get(self.point_idx - 1)
                .map(|point| point.time)
                .unwrap_or(f32::NEG_INFINITY)
        } else {
            f32::NEG_INFINITY
        };
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
            new_easing: self
                .new_easing
                .map(|new| replace(&mut point.ease_type, new)),
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        let canvas_len = chart.canvases.len();
        if let Some(canvas) = self.new_canvas {
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
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
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
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.line_path.valid(chart)
    }
}

#[derive(Debug, Clone)]
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
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        let points_len = self.line_path.get(chart)?.points.len();
        if points_len < self.point_idx {
            Err(ChartConflictError::NoSuchPoint {
                line_path: self.line_path,
                point: self.point_idx,
            })
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct InsertLineColorPoint {
    pub point: KeyPoint<ColorRGBA>,
    pub at: Option<usize>,
    pub line_index: usize,
}

impl ChartCommand for InsertLineColorPoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let line = chart.lines.get_mut(self.line_index).ok_or(crate::editing::ChartConflictError::InvalidLinePath { line_path: self.line_index.into() })?;
        let len = line.line_color.points().len();
        let at = self.at.unwrap_or(len);
        if at > len {
             return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
        }
        line.line_color.points.insert(at, self.point.clone());
        Ok(RemoveLineColorPoint {
            path: LineColorPath::new(self.line_index, at),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        let line = chart.lines.get(self.line_index).ok_or(crate::editing::ChartConflictError::InvalidLinePath { line_path: self.line_index.into() })?;
        let len = line.line_color.points().len();
        if let Some(at) = self.at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RemoveLineColorPoint {
    pub path: LineColorPath,
}

impl ChartCommand for RemoveLineColorPoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let point = self.path.remove(chart)?;
        Ok(InsertLineColorPoint {
            point,
            at: Some(self.path.1),
            line_index: self.path.0,
        }.into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.path.valid(chart)
    }
}

#[derive(Debug, Clone)]
pub struct EditLineColorPoint {
    pub path: LineColorPath,
    pub new_time: Option<f32>,
    pub new_value: Option<ColorRGBA>,
    pub new_easing: Option<EasingId>,
}

impl ChartCommand for EditLineColorPoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let point = self.path.get_mut(chart)?;
        let current_time = point.time;
        let old_time = replace(&mut point.time, self.new_time.unwrap_or(current_time));
        let current_value = point.value;
        let old_value = replace(&mut point.value, self.new_value.unwrap_or(current_value));
        let current_easing = point.ease_type;
        let old_easing = replace(&mut point.ease_type, self.new_easing.unwrap_or(current_easing));
        
        Ok(EditLineColorPoint {
            path: self.path,
            new_time: Some(old_time),
            new_value: Some(old_value),
            new_easing: Some(old_easing),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.path.valid(chart)
    }
}

#[derive(Debug, Clone)]
pub struct InsertRingColorPoint {
    pub point: KeyPoint<ColorRGBA>,
    pub at: Option<usize>,
    pub line_index: usize,
}

impl ChartCommand for InsertRingColorPoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let line = chart.lines.get_mut(self.line_index).ok_or(crate::editing::ChartConflictError::InvalidLinePath { line_path: self.line_index.into() })?;
        let len = line.ring_color.points().len();
        let at = self.at.unwrap_or(len);
        if at > len {
             return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
        }
        line.ring_color.points.insert(at, self.point.clone());
        Ok(RemoveRingColorPoint {
            path: RingColorPath::new(self.line_index, at),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        let line = chart.lines.get(self.line_index).ok_or(crate::editing::ChartConflictError::InvalidLinePath { line_path: self.line_index.into() })?;
        let len = line.ring_color.points().len();
        if let Some(at) = self.at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RemoveRingColorPoint {
    pub path: RingColorPath,
}

impl ChartCommand for RemoveRingColorPoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let point = self.path.remove(chart)?;
        Ok(InsertRingColorPoint {
            point,
            at: Some(self.path.1),
            line_index: self.path.0,
        }.into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.path.valid(chart)
    }
}

#[derive(Debug, Clone)]
pub struct EditRingColorPoint {
    pub path: RingColorPath,
    pub new_time: Option<f32>,
    pub new_value: Option<ColorRGBA>,
    pub new_easing: Option<EasingId>,
}

impl ChartCommand for EditRingColorPoint {
    fn apply(self, chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
        let point = self.path.get_mut(chart)?;
        let current_time = point.time;
        let old_time = replace(&mut point.time, self.new_time.unwrap_or(current_time));
        let current_value = point.value;
        let old_value = replace(&mut point.value, self.new_value.unwrap_or(current_value));
        let current_easing = point.ease_type;
        let old_easing = replace(&mut point.ease_type, self.new_easing.unwrap_or(current_easing));
        
        Ok(EditRingColorPoint {
            path: self.path,
            new_time: Some(old_time),
            new_value: Some(old_value),
            new_easing: Some(old_easing),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> crate::editing::Result<()> {
        self.path.valid(chart)
    }
}

