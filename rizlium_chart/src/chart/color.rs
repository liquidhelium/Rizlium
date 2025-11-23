use crate::tween;

use super::Tween;
use std::ops::Add;

#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;

/// 线性 srgba, 每个值都在 `0.0..=1.0` 内.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
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
        let mut blend = ColorRGBA {
            r: self.r * (1.0 - rhs.a) + rhs.r * rhs.a,
            g: self.g * (1.0 - rhs.a) + rhs.g * rhs.a,
            b: self.b * (1.0 - rhs.a) + rhs.b * rhs.a,
            a: self.a * (1.0 - rhs.a) + rhs.a * rhs.a,
        };
        blend.a = self.a;
        blend
    }
}
impl ColorRGBA {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const fn from_u8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self {
            r: r as f32 / 255.,
            g: g as f32 / 255.,
            b: b as f32 / 255.,
            a: a as f32 / 255.,
        }
    }
    pub const fn from_rgb_u8(r: u8, g: u8, b: u8) -> Self {
        Self::from_u8(r, g, b, 255)
    }
}
