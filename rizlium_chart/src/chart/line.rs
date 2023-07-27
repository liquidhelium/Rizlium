use super::*;

/// 核心谱面元素: 线, 包含所有 [`Note`].
#[derive(Debug, Clone)]
pub struct Line {
    pub points: Spline<f32,usize>,
    pub point_color: Spline<ColorRGBA,usize>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
}
