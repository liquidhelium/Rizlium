use crate::tween;

use super::{ColorRGBA, Tween};
#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;

macro_rules! fields_str {
    ($this:ident, $match:ident,$($str:ident),+) => {
        match $match {
            $(stringify!($str) => Some($this.$str),)+
            _ => None
        }
    };
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct ThemeData {
    pub color: ThemeColor,
    pub is_challenge: bool,
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct ThemeColor {
    pub background: ColorRGBA,
    pub note: ColorRGBA,
    pub fx: ColorRGBA,
}
impl ThemeColor {
    pub fn color_with_id(&self, id: &str) -> Option<ColorRGBA> {
        fields_str!(self, id, background, note)
    }
}

#[derive(Debug)]
pub struct ThemeTransition<'a> {
    pub this: &'a ThemeData,
    pub next: &'a ThemeData,
    pub progress: f32,
}

impl Tween for ThemeColor {
    fn lerp(x1: Self, x2: Self, t: f32) -> Self {
        tween!((background, note, fx), x1, x2, t)
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
