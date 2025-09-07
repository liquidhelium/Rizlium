// 声明并公开 `command_panel` 模块，提供命令面板功能。
pub mod command_panel;
// 声明并公开 `debug_flycam` 模块，提供调试用的自由飞行相机功能。
pub mod debug_flycam;
// 声明并公开 `docking` 模块，管理编辑器中的窗口停靠系统。
pub mod docking;
// 声明 `editing` 模块，包含核心的谱面编辑功能。此模块为私有，其公共接口可能通过其他方式暴露。
mod editing;
// 声明 `game` 模块，包含游戏运行时的相关逻辑。此模块为私有。
mod game;
// 声明并公开 `i18n` 模块，用于处理国际化和本地化（多语言支持）。
pub mod i18n;
// 声明 `inspector` 模块，提供一个可以查看和编辑 Bevy 实体组件数据的检查器窗口。此模块为私有。
mod inspector;
// 声明并公开 `explorer` 模块，可能用于文件或项目内容的浏览。
pub mod explorer;


// 引入 Bevy 引擎的应用（App）和插件（Plugin）核心 trait。
use bevy::prelude::{App, Plugin};


// 引入 `debug_flycam` 模块中的 `DebugCamExtension` 插件。
use crate::extensions::debug_flycam::DebugCamExtension;

// 使用 `self` 关键字来从当前模块（`extensions`）的子模块中导入多个项。
// 这种写法可以避免重复写 `crate::extensions::` 前缀，使代码更简洁。
use self::{
    command_panel::CommandPanel, // 命令面板插件
    docking::Docking,           // 停靠系统插件
    editing::Editing,           // 编辑功能插件
    game::Game,                 // 游戏逻辑插件
    i18n::I18nPlugin,           // 国际化插件
    inspector::Inspector,       // 检查器插件
};

/// `ExtensionsPlugin` 是一个聚合插件（Aggregate Plugin）。
/// 它的唯一作用就是将所有编辑器相关的扩展功能插件一次性地添加到 Bevy 应用中。
/// 这样做可以简化主程序的设置过程，将所有扩展的管理集中到这个文件里。
pub struct ExtensionsPlugin;

// 为 `ExtensionsPlugin` 实现 Bevy 的 `Plugin` trait。
impl Plugin for ExtensionsPlugin {
    /// `build` 方法是 `Plugin` trait 的核心。
    /// 当这个插件被添加到 `App` 时，Bevy 会调用这个方法。
    ///
    /// # 参数
    /// * `app`: 一个对 `App` 的可变引用，用于注册插件、系统、资源等。
    fn build(&self, app: &mut App) {
        // `app.add_plugins` 方法可以一次性添加多个插件。
        // 这里我们将所有独立的扩展插件作为一个元组传入。
        // 插件的添加顺序有时很重要，因为它会影响系统的执行顺序。
        app.add_plugins((
            I18nPlugin,          // 添加国际化插件
            Game,                // 添加游戏逻辑插件
            Docking,             // 添加停靠系统插件
            CommandPanel,        // 添加命令面板插件
            Editing,             // 添加编辑功能插件
            Inspector,           // 添加检查器插件
            DebugCamExtension,   // 添加调试相机插件
        ));
    }
}