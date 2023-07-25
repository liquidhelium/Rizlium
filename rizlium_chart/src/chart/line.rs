use super::*;

#[derive(Debug, Clone)]
pub struct Line {
    /// Related to canvas
    pub points: Spline<f32>,
    /// Related to `line_color`
    pub point_color: Spline<ColorRGBA>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
}

