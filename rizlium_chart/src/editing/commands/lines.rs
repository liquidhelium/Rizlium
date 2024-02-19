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
}

pub struct MovePoint {
    pub line_path: LinePath,
    pub point_idx: usize,
    pub new_time: f32,
    pub new_x: f32,
    pub new_canvas: Option<usize>,
}

impl ChartCommand for MovePoint {
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

        self.new_time = self.new_time.clamp(prev_time, next_time);
        // swap 也许好一点
        let old_time = point.time;
        let old_x = point.value;
        point.time = self.new_time;
        point.value = self.new_x;
        Ok(Self {
            line_path: self.line_path,
            point_idx: self.point_idx,
            new_time: old_time,
            new_x: old_x,
            new_canvas: old_canvas,
        }
        .into())
    }
}

pub struct InsertPoint {
    pub line_path: LinePath,
    pub point_idx: Option<usize>,
    pub point: KeyPoint<f32, usize>
}

// impl ChartCommand for InsertPoint {
//     fn apply(self,chart: &mut Chart) -> crate::editing::Result<super::ChartCommands> {
//         let canvas_len = chart.canvases.len();
//         let line = self.line_path.get_mut(chart)?;
//         let at = self.point_idx.unwrap_or(line.points.len()).clamp(0, line.points.len());
//         let prev_time = line
//             .points
//             .points
//             .get(self.point_idx - 1)
//             .map(|point| point.time)
//             .unwrap_or(f32::NEG_INFINITY);
//         let next_time = line
//             .points
//             .points
//             .get(self.point_idx + 1)
//             .map(|point| point.time)
//             .unwrap_or(f32::INFINITY);
//     }
// }
