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

impl Line {
    pub fn new_two_points(pos1: [f32; 2], pos2: [f32; 2]) -> Self {
        let (former, latter) = if pos1[0] > pos2[0] {
            (pos2, pos1)
        } else {
            (pos1, pos2)
        };
        Self {
            points: Spline {
                points: vec![
                    KeyPoint::from_slice(
                        former,
                        EasingId::Linear,
                        LinePointData {
                            canvas: 0,
                            color: ColorRGBA::BLACK,
                        },
                    ),
                    KeyPoint::from_slice(latter, EasingId::Linear, LinePointData {
                        canvas: 0,
                        color: ColorRGBA::BLACK,
                    }),
                ],
            },
            line_color: Spline::EMPTY,
            notes: vec![],
            ring_color: Spline::EMPTY,
        }
    }
}
