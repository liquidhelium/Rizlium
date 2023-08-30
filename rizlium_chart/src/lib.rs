//! # Rizlium 谱面格式
//! Rizlium 所使用的谱面结构及部分操作.

/// 核心数据结构.
pub mod chart;
/// 从文件创建Rizlium谱面.
pub mod parse;

/// 在某个时间的谱面状态.
#[cfg(feature = "runtime")]
pub mod runtime;

#[cfg(feature = "editing")]
pub mod editing;

/// 正常情况下游戏画面截取的部分.
pub const VIEW_RECT: [[f32; 2]; 2] = [[-450., 0.], [450., 1600.]];

pub mod prelude {
    pub use super::chart::*;
    pub use super::parse::official::RizlineChart;
    #[cfg(feature = "runtime")]
    pub use super::runtime::*;
}
