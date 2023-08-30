use super::*;

#[cfg(feature = "serialize")]
use serde::Serialize;
#[cfg(feature = "deserialize")]
use serde::Deserialize;

/// 核心谱面元素: 线, 包含所有 [`Note`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct Line {
    pub points: Spline<f32, usize>,
    pub point_color: Spline<ColorRGBA, usize>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
}
