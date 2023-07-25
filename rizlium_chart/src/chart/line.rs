use super::*;

/// Chart element where [`Note`]s lay on.
#[derive(Debug, Clone)]
pub struct Line {
    pub points: Spline<f32>,
    pub point_color: Spline<ColorRGBA>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
}

