use super::*;

#[derive(Debug, Clone)]
pub struct LineNext {
    /// Related to canvas
    pub points: SplineNext<f32>,
    /// Related to `line_color`
    pub point_color: SplineNext<ColorRGBA>,
    pub notes: Vec<Note>,
    pub ring_color: SplineNext<ColorRGBA>,
    pub line_color: SplineNext<ColorRGBA>,
}

