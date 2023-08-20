use crate::tween;

use super::Tween;
use std::ops::Add;

/// 线性 srgba, 每个值都在 `0.0..=1.0` 内.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Tween for ColorRGBA {
    fn lerp(x1: Self, x2: Self, t: f32) -> Self {
        tween!((r, g, b, a), x1, x2, t)
    }
}
impl Add for ColorRGBA {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        // blend: (ONE_MINUS_SRC_ALPHA, SRC_ALPHA)
        let mut blend = Self::lerp(rhs, self, rhs.a / 255.);
        blend.a = self.a;
        blend
    }
}
