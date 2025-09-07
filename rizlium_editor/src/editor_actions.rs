// 引入项目中定义的模块，用于处理谱面加载、项目状态和最近文件列表
use crate::project::{LoadChartEvent, ProjectState, RecentFiles};
// 引入 Bevy 引擎的 ECS（实体组件系统）相关模块，用于系统参数、系统元信息和命令队列
use bevy::ecs::system::{SystemBuffer, SystemMeta, SystemParam};
// 引入 Bevy 引擎的世界命令队列
use bevy::ecs::world::CommandQueue;
// 引入 Bevy 引擎的核心预设模块
use bevy::prelude::*;
// 引入 Bevy 的持久化存储插件，用于保存数据
use bevy_persistent::Persistent;
// 引入渲染模块中的 ShowLines 资源，用于控制谱面线的显示
use rizlium_render::ShowLines;
// 引入时间和音频控制事件模块
use crate::time_and_audio::TimeControlEvent;
// 引入 Serde 库，用于序列化和反序列化
use serde::de::DeserializeOwned;
use serde::Serialize;

/// EditorCommands 是一个 Bevy 系统参数（SystemParam），
/// 它封装了 `ManualEditorCommands` 的延迟执行版本。
/// 这样做可以将命令的执行推迟到系统更新的特定阶段（通常是末尾），
/// 从而避免在系统内部直接修改 `World` 时可能引发的冲突和数据竞争。
/// 使用 `Deref` 和 `DerefMut` 可以让 `EditorCommands` 像 `ManualEditorCommands` 一样被直接调用。
#[derive(SystemParam, Deref, DerefMut)]
pub struct EditorCommands<'s> {
    // `Deferred` 是 Bevy 提供的一种机制，用于延迟对系统参数的操作。
    // 这里它包装了 `ManualEditorCommands`，意味着所有通过 `EditorCommands` 发出的命令
    // 都会被暂存起来，在适当的时机由 Bevy 的调度器统一应用。
    commands: Deferred<'s, ManualEditorCommands>,
}

/// ManualEditorCommands 是一个手动管理的命令队列。
/// 它允许在 Bevy 的 ECS 系统之外（例如在 UI 回调或异步任务中）创建和存储命令，
/// 然后在需要时手动将这些命令应用到 `World` 中。
#[derive(Default)]
pub struct ManualEditorCommands {
    // `CommandQueue` 是 Bevy 用来存储一系列待执行操作（命令）的数据结构。
    // 每个命令都是一个闭包，它接收一个可变的 `World` 引用作为参数。
    commands: CommandQueue,
}

// 为 `ManualEditorCommands` 实现 `SystemBuffer` trait。
// 这使得 `ManualEditorCommands` 可以被 Bevy 的 `Deferred` 系统识别和处理，
// 从而能够融入 Bevy 的延迟命令执行流程中。
impl SystemBuffer for ManualEditorCommands {
    /// `apply` 方法定义了如何将缓存的命令应用到 `World`。
    /// 当 Bevy 的调度器处理 `Deferred<ManualEditorCommands>` 时，会调用此方法。
    fn apply(&mut self, _system_meta: &SystemMeta, world: &mut World) {
        // 直接调用内部 `CommandQueue` 的 `apply` 方法，执行所有已入队的命令。
        self.commands.apply(world);
    }
}

impl ManualEditorCommands {
    /// 向命令队列中添加一个发送 `TimeControlEvent` 的命令。
    /// 这个命令将在队列被应用时，向 `World` 中发送一个时间控制事件。
    ///
    /// # 参数
    /// * `event`: 要发送的时间控制事件。
    pub fn time_control(&mut self, event: TimeControlEvent) {
        // `self.commands.push` 将一个闭包（即一个命令）添加到队列中。
        // 这个闭包捕获了 `event` 变量。
        self.commands.push(|world: &mut World| {
            // 当命令执行时，调用 `world.send_event` 来发送事件。
            world.send_event(event);
        });
    }

    /// 向命令队列中添加一个加载谱面的命令。
    ///
    /// # 参数
    /// * `path`: 要加载的谱面文件的路径。
    pub fn load_chart(&mut self, path: String) {
        // 克隆路径字符串，因为 `path` 的所有权需要被移动到 `update_recent` 方法中。
        let dup = path.clone();
        // 添加发送 `LoadChartEvent` 的命令。
        self.commands.push(|world: &mut World| {
            // 发送事件，请求加载指定的谱面包。
            world.send_event(LoadChartEvent::Bundle(dup));
        });
        // 调用另一个方法，将当前加载的路径更新到“最近文件”列表中。
        self.update_recent(path);
    }

    /// 向命令队列中添加一个打开文件对话框以加载谱面的命令。
    pub fn open_dialog_and_load_chart(&mut self) {
        // 添加一个命令，该命令会获取 `ProjectState` 资源并调用其方法。
        self.commands.push(|world: &mut World| {
            // 从 `World` 中可变地借用 `ProjectState` 资源。
            let mut state = world.resource_mut::<ProjectState>();
            // 调用方法以触发操作系统的文件打开对话框。
            state.open_bundle_dialog();
        });
    }

    /// 向命令队列中添加一个更新“最近文件”列表的命令。
    ///
    /// # 参数
    /// * `path`: 要添加到最近文件列表中的文件路径。
    pub fn update_recent(&mut self, path: String) {
        // `move` 关键字将 `path` 的所有权转移到闭包中。
        self.commands.push(move |world: &mut World| {
            // 获取被 `Persistent` 包装的 `RecentFiles` 资源。
            // `Persistent` 是一个包装器，用于自动处理资源的加载和保存。
            let mut recent = world.resource_mut::<Persistent<RecentFiles>>();
            // 将新路径添加到列表中。
            recent.push(path);
            // `persist()` 方法会将 `RecentFiles` 的当前状态保存到磁盘。
            // `.unwrap()` 用于处理可能的错误，如果保存失败则会 panic。
            recent.persist().unwrap();
        });
    }

    /// 向命令队列中添加一个通用的、用于持久化任何资源的命令。
    ///
    /// # 类型参数
    /// * `T`: 必须是 `Resource`（能作为 Bevy 资源）、`Serialize`（可序列化）和 `DeserializeOwned`（可反序列化）的类型。
    pub fn persist_resource<T: Resource + Serialize + DeserializeOwned>(&mut self) {
        self.commands.push(|world: &mut World| {
            // 获取指定的泛型资源 `T`，并调用 `persist` 方法将其保存。
            world.resource_mut::<Persistent<T>>().persist().unwrap();
        });
    }

    /// 手动应用所有已入队的命令。
    /// 这在不通过 Bevy 系统调度器，而是想立即执行命令时很有用。
    ///
    /// # 参数
    /// * `world`: 要应用命令的目标 `World`。
    pub fn apply_manual(&mut self, world: &mut World) {
        self.commands.apply(world);
    }
}

/// GameConfigure 是一个辅助结构体，用于以链式调用的方式配置游戏相关参数。
/// 这种设计模式（通常称为建造者模式或流式接口）使得配置代码更具可读性。
pub struct GameConfigure<'c> {
    // 持有一个对 `ManualEditorCommands` 的可变引用，以便将配置操作作为命令入队。
    pub commands: &'c mut ManualEditorCommands,
}

impl GameConfigure<'_> {
    /// 配置是否显示谱面线以及显示哪一条。
    ///
    /// # 参数
    /// * `show`: 一个 `Option<usize>`。
    ///   - `Some(index)`: 显示索引为 `index` 的谱面线。
    ///   - `None`: 隐藏所有谱面线。
    ///
    /// # 返回值
    /// 返回 `Self`，以支持链式调用。
    pub fn show_line(self, show: Option<usize>) -> Self {
        // 将修改 `ShowLines` 资源的逻辑作为一个命令推入队列。
        // `move` 关键字将 `show` 的所有权转移到闭包中。
        self.commands.commands.push(move |world: &mut World| {
            // 尝试从 `World` 中获取 `ShowLines` 资源的可变引用。
            if let Some(mut res) = world.get_resource_mut::<ShowLines>() {
                // 如果成功获取，则更新其值。
                res.0 = show
            } else {
                // 如果 `ShowLines` 资源不存在，则记录一个错误。
                // 这种情况通常不应该发生，除非初始化有问题。
                error!("failed to get resource!")
            }
        });
        // 返回自身，允许继续调用其他配置方法，例如 `self.another_config().yet_another_config()`。
        self
    }
}
