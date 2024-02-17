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
    pub points: Spline<f32, usize>,
    pub point_color: Spline<ColorRGBA>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub line_color: Spline<ColorRGBA>,
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
                    KeyPoint::from_slice(former, EasingId::Linear, 0),
                    KeyPoint::from_slice(latter, EasingId::Linear, 0),
                ],
            },
            line_color: Spline::EMPTY,
            notes: vec![],
            point_color: Spline {
                points: vec![
                    KeyPoint {
                        time: former[0],
                        value: ColorRGBA::BLACK,
                        ease_type: EasingId::Linear,
                        relevant: ()
                    },
                    KeyPoint {
                        time: latter[0],
                        value: ColorRGBA::BLACK,
                        ease_type: EasingId::Linear,
                        relevant: ()
                    }
                ]
            },
            ring_color: Spline::EMPTY,
        }
    }
}
