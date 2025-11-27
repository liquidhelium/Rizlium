use std::borrow::Cow;

use crate::editing::chart_path::{CanvasPath, CanvasSpeedPath, CanvasXPosPath, ChartPath as _};
use crate::prelude::{Chart, EasingId, KeyPoint};
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct InsertCanvasXPosPoint {
    pub point: KeyPoint<f32>,
    pub at: Option<usize>,
    pub canvas_index: usize,
}

impl super::ChartCommand for InsertCanvasXPosPoint {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let canvas = chart.canvases.get_mut(self.canvas_index).ok_or(crate::editing::ChartConflictError::NoSuchCanvas { canvas: self.canvas_index })?;
        let len = canvas.x_pos.points.len();
        let at = self.at.unwrap_or(len);
        if at > len {
             return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
        }
        canvas.x_pos.points.insert(at, self.point);
        Ok(RemoveCanvasXPosPoint {
            path: CanvasXPosPath::new(self.canvas_index, at),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        let canvas = chart.canvases.get(self.canvas_index).ok_or(crate::editing::ChartConflictError::NoSuchCanvas { canvas: self.canvas_index })?;
        let len = canvas.x_pos.points.len();
        if let Some(at) = self.at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
            }
        }
        Ok(())
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Insert Canvas {} XPos Point at {}", self.canvas_index, self.at.map(|x| x.to_string()).unwrap_or("end".into())).into()
    }
}

#[derive(Debug, Clone)]
pub struct RemoveCanvasXPosPoint {
    pub path: CanvasXPosPath,
}

impl super::ChartCommand for RemoveCanvasXPosPoint {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let removed = self.path.remove(chart)?;
        Ok(InsertCanvasXPosPoint {
            point: removed,
            at: Some(self.path.1),
            canvas_index: self.path.0,
        }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        self.path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Remove Canvas {} XPos Point {}", self.path.0, self.path.1).into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditCanvasXPosPoint {
    pub path: CanvasXPosPath,
    pub new_time: Option<f32>,
    pub new_value: Option<f32>,
    pub new_easing: Option<EasingId>,
}

impl super::ChartCommand for EditCanvasXPosPoint {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let point = self.path.get_mut(chart)?;
        let old_time = point.time;
        let old_value = point.value;
        let old_easing = point.ease_type;

        if let Some(t) = self.new_time { point.time = t; }
        if let Some(v) = self.new_value { point.value = v; }
        if let Some(e) = self.new_easing { point.ease_type = e; }

        Ok(EditCanvasXPosPoint {
            path: self.path,
            new_time: self.new_time.map(|_| old_time),
            new_value: self.new_value.map(|_| old_value),
            new_easing: self.new_easing.map(|_| old_easing),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        self.path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Edit Canvas {} XPos Point {}", self.path.0, self.path.1).into()
    }
}

#[derive(Debug, Clone)]
pub struct InsertCanvasSpeedPoint {
    pub point: KeyPoint<f32>,
    pub at: Option<usize>,
    pub canvas_index: usize,
}

impl super::ChartCommand for InsertCanvasSpeedPoint {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let canvas = chart.canvases.get_mut(self.canvas_index).ok_or(crate::editing::ChartConflictError::NoSuchCanvas { canvas: self.canvas_index })?;
        let len = canvas.speed.points.len();
        let at = self.at.unwrap_or(len);
        if at > len {
             return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
        }
        canvas.speed.points.insert(at, self.point);
        Ok(RemoveCanvasSpeedPoint {
            path: CanvasSpeedPath::new(self.canvas_index, at),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        let canvas = chart.canvases.get(self.canvas_index).ok_or(crate::editing::ChartConflictError::NoSuchCanvas { canvas: self.canvas_index })?;
        let len = canvas.speed.points.len();
        if let Some(at) = self.at {
            if at > len {
                return Err(crate::editing::ChartConflictError::IndexOutOfBounds { index: at, len });
            }
        }
        Ok(())
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Insert Canvas {} Speed Point at {}", self.canvas_index, self.at.map(|x| x.to_string()).unwrap_or("end".into())).into()
    }
}

#[derive(Debug, Clone)]
pub struct RemoveCanvasSpeedPoint {
    pub path: CanvasSpeedPath,
}

impl super::ChartCommand for RemoveCanvasSpeedPoint {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let removed = self.path.remove(chart)?;
        Ok(InsertCanvasSpeedPoint {
            point: removed,
            at: Some(self.path.1),
            canvas_index: self.path.0,
        }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        self.path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Remove Canvas {} Speed Point {}", self.path.0, self.path.1).into()
    }
}

#[derive(Debug, Clone, Default)]
pub struct EditCanvasSpeedPoint {
    pub path: CanvasSpeedPath,
    pub new_time: Option<f32>,
    pub new_value: Option<f32>,
    pub new_easing: Option<EasingId>,
}

impl super::ChartCommand for EditCanvasSpeedPoint {
    fn apply(self, chart: &mut Chart) -> super::Result<super::ChartCommands> {
        let point = self.path.get_mut(chart)?;
        let old_time = point.time;
        let old_value = point.value;
        let old_easing = point.ease_type;

        if let Some(t) = self.new_time { point.time = t; }
        if let Some(v) = self.new_value { point.value = v; }
        if let Some(e) = self.new_easing { point.ease_type = e; }

        Ok(EditCanvasSpeedPoint {
            path: self.path,
            new_time: self.new_time.map(|_| old_time),
            new_value: self.new_value.map(|_| old_value),
            new_easing: self.new_easing.map(|_| old_easing),
        }.into())
    }
    fn validate(&self, chart: &Chart) -> super::Result<()> {
        self.path.valid(chart)
    }
    fn description(&self) -> Cow<'static, str> {
        format!("Edit Canvas {} Speed Point {}", self.path.0, self.path.1).into()
    }
}
