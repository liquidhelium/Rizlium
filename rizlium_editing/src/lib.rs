//! Rizlium Editing - 编辑功能扩展
//!
//! 这个crate提供了编辑功能扩展，
//! 包括音符编辑器、样条编辑器、时间线编辑器、工具配置和选择、撤销/重做功能以及世界视图。

pub mod extensions;

pub use extensions::*;

// 初始化i18n
rust_i18n::i18n!("locales");