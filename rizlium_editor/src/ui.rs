// 声明并公开 `widgets` 子模块，其中包含自定义的 egui 小部件。
pub mod widgets;
// 声明并公开 `theme` 子模块，其中包含 egui 的主题和样式定义。
pub mod theme;

// 引入 Bevy 引擎的相关模块。
use bevy::{
    asset::uuid::Uuid, // 引入 UUID 类型，用于唯一标识预设。
    ecs::{
        change_detection::{DetectChanges, DetectChangesMut}, // 引入变更检测相关的 trait。
        schedule::Condition, // 引入 `Condition` trait，用于创建 Bevy 系统的运行条件。
        system::{Res, ResMut}, // 引入对资源的只读和可变访问类型。
    },
    log::debug, // 引入 `debug!` 日志宏。
    prelude::{Deref, DerefMut, Resource}, // 引入常用的 trait 和宏。
};

// 引入 `bevy_persistent` 插件，用于持久化数据。
use bevy_persistent::Persistent;
// 引入 `egui-dock` 库的核心数据结构。
use egui_dock::{DockState, Tree};

// 引入 `helium_framework` 中的 `TabId` 类型。
use helium_framework::prelude::TabId;
// 引入 `serde` 的 `Deserialize` 和 `Serialize` trait，用于序列化和反序列化。
use serde::{Deserialize, Serialize};

/// `RizTabPresets` 是一个 Bevy 资源，用于存储用户保存的标签页布局预设。
/// 它是一个元组的向量，每个元组包含一个唯一的 UUID、一个用户指定的名称和该布局的 `DockState`。
#[derive(Resource, Serialize, Deserialize, Default, DerefMut, Deref, Clone)]
pub struct RizTabPresets(Vec<(Uuid, String, DockState<TabId>)>);

/// `RizliumDockStateMirror` 是一个 Bevy 资源，它作为 `RizliumDockState` 的“镜像”或“影子副本”。
/// 它的主要目的是解决 Bevy 的变更检测系统和 `egui-dock` 内部状态修改之间的冲突。
/// `egui-dock` 会在渲染时直接修改传入的 `DockState`，但这不会被 Bevy 的变更检测系统自动捕捉到。
/// 通过引入一个镜像，我们可以手动比较两者，并在需要时触发同步，从而确保状态的一致性和正确的持久化。
#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct RizliumDockStateMirror(pub Option<DockState<TabId>>);

/// `sync_dock_state` 是一个 Bevy 系统，负责在持久化的 `RizliumDockState` 和其镜像 `RizliumDockStateMirror` 之间同步数据。
/// 这个系统应该在 `RizliumDockState` 或 `RizliumDockStateMirror` 可能发生变化之后运行。
pub fn sync_dock_state(
    mut dock_state: ResMut<Persistent<RizliumDockState>>,
    mut mirror: ResMut<RizliumDockStateMirror>,
) {
    // 检查是否是镜像发生了变化。这通常意味着 `egui-dock` 在 UI 线程中更新了布局。
    if mirror.is_changed() {
        // 如果镜像有新的状态，则将其同步到持久化的 `dock_state` 中。
        if let Some(mirror_state) = &mirror.0 {
            debug!("Syncing mirror state");
            // `bypass_change_detection()` 用于防止这次赋值本身再次触发变更检测，避免无限循环。
            dock_state.bypass_change_detection().0 = mirror_state.clone();
        }
    } else {
        // 否则，认为是持久化的 `dock_state` 发生了变化（例如，从文件加载或被其他系统修改）。
        // 此时，我们将 `dock_state` 的内容同步到镜像中，以供下一帧 UI 渲染使用。
        mirror.bypass_change_detection().0 = Some(dock_state.0.clone());
    }
}

/// `RizliumDockState` 是一个 Bevy 资源，它包装了 `egui_dock` 的核心状态结构 `DockState<TabId>`。
/// 这个资源会被 `bevy_persistent` 插件自动保存到磁盘，并在下次启动时加载，从而实现布局的持久化。
#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RizliumDockState(pub DockState<TabId>);

impl Default for RizliumDockState {
    fn default() -> Self {
        // 创建一个空的 `DockState`。
        // 这是一个有点 hack 的方法：`egui-dock` 的 `DockState::new` 需要一个初始的标签列表，
        // 但我们希望初始时是完全空的。
        let mut dock_state = DockState::new(vec![]);

        // 通过直接替换主表面为一个空的 `Tree`，我们得到了一个真正没有任何窗口或标签的停靠状态。
        // 这使得编辑器在首次启动或没有加载任何标签页时，可以显示一个欢迎界面而不是一个空的停靠区域。
        *dock_state.main_surface_mut() = Tree::default();
        Self(dock_state)
    }
}
/// `tab_opened` 是一个工厂函数，用于创建一个 Bevy 系统运行条件（`Condition`）。
/// 这个条件会在指定的标签页 ID 当前处于打开状态时返回 `true`。
///
/// # 参数
/// * `tab`: 要检查的标签页的 ID。
///
/// # 示例
/// ```rust,ignore
/// app.add_systems(Update, my_system.run_if(tab_opened("my_tab_id")));
/// ```
pub fn tab_opened(tab: impl Into<TabId>) -> impl Condition<()> {
    let tab = tab.into();
    // 返回一个闭包，该闭包捕获了 `tab` ID。
    // Bevy 会将这个闭包作为一个系统来运行，以判断条件是否满足。
    (move |res: Option<Res<RizliumDockStateMirror>>| {
        // 检查 `RizliumDockStateMirror` 资源是否存在，并且其内部的 `DockState` 是否包含指定的标签页。
        res.is_some_and(|res| res.0.as_ref().is_some_and(|r| r.find_tab(&tab).is_some()))
    })
    // `.and(|| true)` 是一个必要的技巧，用于帮助 Rust 的类型推断系统正确推断闭包的类型。
    .and(|| true)
}
