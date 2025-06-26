use std::mem::replace;

use super::*;

#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;

/// 核心谱面元素: 线, 包含所有 [`Note`].
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct Line {
    pub points: Spline<f32, LinePointData>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
}

/// 线上的点的相关数据.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct LinePointData {
    pub canvas: usize,
    pub color: ColorRGBA,
}

impl FromIterator<KeyPoint<f32, LinePointData>> for Line {
    fn from_iter<T: IntoIterator<Item = KeyPoint<f32, LinePointData>>>(iter: T) -> Self {
        let mut points: Vec<_> = iter.into_iter().collect();
        points
            .iter_mut()
            .fold(f32::NEG_INFINITY, |lower_limit, point| {
                let src = point.time.max(lower_limit);
                replace(&mut point.time, src)
            });
        Self {
            points: Spline { points },
            notes: vec![],
            ring_color: Spline::EMPTY,
            line_color: Spline::EMPTY,
        }
    }
}
