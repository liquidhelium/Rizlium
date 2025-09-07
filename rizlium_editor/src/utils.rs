// 引入 Bevy ECS 相关的模块。
use bevy::ecs::{
    schedule::{BoxedCondition, Condition}, // `Condition` 用于创建系统运行条件，`BoxedCondition` 是其动态分派版本。
    system::{IntoSystem, Res, System, SystemParam}, // `SystemParam` 用于创建自定义的系统参数。
};
// 引入渲染模块中的 `GameChartCache` 和 `GameTime` 资源。
use rizlium_render::{GameChartCache, GameTime};

/// `WorldToGame` 是一个自定义的 Bevy 系统参数（SystemParam）。
/// 它将谱面渲染时常用的资源（`GameChartCache` 和 `GameTime`）打包在一起，
/// 使得其他系统可以更方便地将世界坐标（world coordinates）转换为游戏时间（game time）。
/// 使用 `Option` 是因为它只在谱面成功加载后才可用。
#[derive(SystemParam)]
pub struct WorldToGame<'w> {
    // 对 `GameChartCache` 资源的可选引用。`GameChartCache` 存储了用于高效计算坐标和时间转换的数据。
    pub cache: Option<Res<'w, GameChartCache>>,
    // 对 `GameTime` 资源的可选引用。`GameTime` 存储了当前的游戏播放时间。
    pub time: Option<Res<'w, GameTime>>,
}

impl WorldToGame<'_> {
    /// 将相对于当前屏幕视图的世界坐标 Y 值转换为游戏时间（通常是节拍数）。
    ///
    /// # 参数
    /// * `world_y`: 世界坐标系中的 Y 值。这个值通常是相对于屏幕中心的，例如鼠标光标的 Y 坐标。
    /// * `canvas`: 渲染画布的索引。
    ///
    /// # 返回值
    /// * `Option<f32>`: 如果转换成功，则返回对应的时间值；如果所需资源不存在，则返回 `None`。
    pub fn time_at_y(&self, world_y: f32, canvas: usize) -> Option<f32> {
        // 1. `self.cache.as_deref()?`: 获取 `GameChartCache` 的引用，如果为 `None` 则提前返回。
        // 2. `self.time.as_deref()?`: 获取 `GameTime` 的引用。
        // 3. `cache.canvas_y_at(canvas, **time)`: 计算当前游戏时间在指定画布上的 Y 坐标。
        // 4. `world_y + ...`: 将输入的相对 Y 坐标 `world_y` 转换为绝对的画布 Y 坐标。
        // 5. `cache.canvas_y_to_time(canvas, ...)`: 将最终的绝对画布 Y 坐标转换为游戏时间。
        self.cache.as_deref()?.canvas_y_to_time(
            canvas,
            world_y
                + self
                    .cache
                    .as_deref()?
                    .canvas_y_at(canvas, **self.time.as_deref()?)?,
        )
    }
    /// 检查 `WorldToGame` 所需的资源是否都已存在。
    pub fn avalible(&self) -> bool {
        self.cache.is_some() && self.time.is_some()
    }
}

/// `new_condition` 是一个辅助函数，用于将任何实现了 `Condition` trait 的类型
/// 转换为一个 `BoxedCondition`（一个动态分派的、可在运行时存储的条件）。
///
/// 注意：这个函数目前有一个限制，即它所包装的条件不能访问任何 `NonSend` 资源。
/// 这是因为 `BoxedCondition` 要求其内部的系统是 `Send` 的。
pub fn new_condition<M>(condition: impl Condition<M>) -> BoxedCondition {
    // 将条件转换为一个系统。
    let condition_system = IntoSystem::into_system(condition);
    // 断言该系统是 `Send` 的。如果不是，程序将在调试模式下 panic，并显示一条有用的错误信息。
    assert!(
        condition_system.is_send(),
        "Condition `{}` accesses `NonSend` resources. This is not currently supported.",
        condition_system.name()
    );

    // 将系统装箱并返回。
    Box::new(condition_system)
}
