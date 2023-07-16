use super::Tween;

macro_rules! tween {
    (($($var:ident),*),$x1:ident,$x2:ident, $t:ident) => {
        Self {
            $($var: f32::tween($x1.$var,$x2.$var,$t),)*
        }
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorRGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Tween for ColorRGBA {
    fn tween(x1: Self, x2: Self, t: crate::chart::Clamped) -> Self {
        tween!((r, g, b, a), x1, x2, t)
    }
}
// todo: impl Add for ColorRBGA {..}
