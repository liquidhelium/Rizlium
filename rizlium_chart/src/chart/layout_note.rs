#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;

use super::NoteKind;

/// 谱面配置中的音符，包含时间和x位置。
/// 这是与line无关的独立数组，用于辅助音符放置。
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct LayoutNote {
    /// 音符的时间（beat）
    pub time: f32,
    /// 音符的x位置（屏幕坐标）
    pub x: f32,
    /// 音符类型
    pub kind: NoteKind,
}

impl LayoutNote {
    pub const fn new(time: f32, x: f32, kind: NoteKind) -> Self {
        Self { time, x, kind }
    }
}

impl PartialEq for LayoutNote {
    fn eq(&self, other: &Self) -> bool {
        self.time == other.time && self.x == other.x && self.kind == other.kind
    }
}
