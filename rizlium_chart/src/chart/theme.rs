use crate::tween;

use super::{ColorRGBA, Tween};

#[derive(Debug, Clone, Copy)]
pub struct ThemeData {
    pub color: ThemeColor,
    pub is_challenge: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ThemeColor {
    pub background: ColorRGBA,
    pub note: ColorRGBA,
}
#[derive(Debug)]
pub struct ThemeTransition<'a> {
    pub this: &'a ThemeData,
    pub next: &'a ThemeData,
    pub progress: f32,
}

impl Tween for ThemeColor {
    fn lerp(x1: Self, x2: Self, t: f32) -> Self {
        tween!((background, note), x1, x2, t)
    }
}

impl Tween for ThemeData {
    fn lerp(x1: Self, x2: Self, t: f32) -> Self {
        Self {
            color: Tween::lerp(x1.color, x2.color, t),
            is_challenge: x1.is_challenge && x2.is_challenge,
        }
    }
}
