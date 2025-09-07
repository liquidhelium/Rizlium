// 引入标准库中的 `Cow`（写时复制智能指针）和 `PhantomData`（零大小标记类型）。
use std::{borrow::Cow, marker::PhantomData};

// 引入 Bevy 引擎的相关模块。
use bevy::{
    app::{App, Plugin}, // App 构建器和插件 trait。
    ecs::{
        resource::Resource, // Resource trait。
        system::{In, InMut, IntoSystem, Local, ReadOnlySystem, System}, // 系统相关的类型和 trait。
        world::{Mut, World}, // World 和可变访问器。
    },
    log::error, // 日志宏。
    prelude::Deref, // Deref trait，用于智能指针解引用。
};
// 引入 egui 库的组件，用于构建 UI。
use egui::{
    Align, Button, CentralPanel, Layout, ScrollArea, SidePanel, Ui, UiBuilder, UiStackInfo,
};
// 引入 `IndexMap`，一个保持插入顺序的哈希映射。
use indexmap::IndexMap;
// 引入国际化宏 `t!`。
use rust_i18n::t;

// 引入自定义的 `helium_framework` 框架。
use helium_framework::{prelude::*, utils::identifier::Identifier};

/// `SettingsRegistrationExt` trait 为 Bevy 的 `App` 类型添加了扩展方法，
/// 以便能够方便地注册新的设置模块。
pub trait SettingsRegistrationExt {
    /// 注册一个新的设置模块。
    ///
    /// # 参数
    /// * `id`: 模块的唯一标识符。
    /// * `module`: 实现了 `SettingsModule` trait 的具体模块实例。
    fn register_settings_module(
        &mut self,
        id: impl Into<Identifier>,
        module: impl SettingsModule,
    ) -> &mut Self;
}

// 为 Bevy 的 `App` 实现 `SettingsRegistrationExt` trait。
impl SettingsRegistrationExt for App {
    fn register_settings_module(
        &mut self,
        id: impl Into<Identifier>,
        module: impl SettingsModule,
    ) -> &mut Self {
        // 将具体的 `SettingsModule` 实现转换为一个动态的、类型擦除的 `SettingsModuleDyn`。
        // 这是通过 `Box::new` 和 `from_module` 实现的，允许我们在注册表中存储不同类型的设置模块。
        let v = Box::new(SettingsModuleDyn::from_module(module, self.world_mut()));
        // 获取 `SettingsModuleRegistry` 资源，并将新的模块插入其中。
        self.world_mut()
            .resource_mut::<SettingsModuleRegistry>()
            .0
            .insert(id.into(), v);
        self
    }
}

/// `SettingsPlugin` 是一个 Bevy 插件，负责初始化设置模块所需的核心资源。
pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    /// `build` 方法在插件被添加到 `App` 时调用。
    fn build(&self, app: &mut bevy::prelude::App) {
        // 初始化 `SettingsModuleRegistry` 资源。如果该资源不存在，则会使用其 `Default` 实现创建一个。
        app.init_resource::<SettingsModuleRegistry>();
    }
    /// `finish` 方法在所有插件的 `build` 方法都执行完毕后调用。
    fn finish(&self, app: &mut App) {
        // 在这里注册“设置”标签页本身。
        // 这样做可以确保 `SettingsModuleRegistry` 已经存在，并且其他插件有机会在 `build` 阶段注册它们的设置模块。
        app.register_tab("settings", t!("settings.tab"), settings_tab, || true);
    }
}

/// `settings_tab` 是一个 Bevy 系统，负责渲染整个设置标签页的 UI。
fn settings_tab(InMut(ui): InMut<Ui>, world: &mut World, mut opened_tab: Local<usize>) {
    // `resource_scope` 提供对 `World` 中资源的临时、安全访问。
    world.resource_scope(
        |world: &mut World, mut registry: Mut<SettingsModuleRegistry>| {
            ui.heading("Settings"); // 显示标题。
            ui.scope(|ui| {
                // 创建一个左侧面板，用于显示所有可用设置模块的列表。
                SidePanel::left("settings_entry")
                    .min_width(60.)
                    .max_width(80.)
                    .show_inside(ui, |ui| {
                        // 使用 `ScrollArea` 以确保在模块过多时可以滚动。
                        ScrollArea::new([false, true]).show(ui, |ui| {
                            // 遍历注册表中的所有模块。
                            for (i, runner) in registry.0.values_mut().enumerate() {
                                // 为每个模块创建一个可选择的标签。
                                if ui
                                    .selectable_label(i == *opened_tab, runner.name())
                                    .clicked()
                                {
                                    // 如果用户点击了某个标签，则更新当前打开的标签页索引。
                                    *opened_tab = i
                                }
                            }
                        });
                    });
                // 创建一个中央面板，用于显示当前选定设置模块的具体 UI。
                CentralPanel::default().show_inside(ui, |ui| {
                    // 根据索引获取当前活动的模块。
                    if let Some((_, runner)) = registry.0.get_index_mut(*opened_tab) {
                        // 运行该模块的 UI 系统来绘制其界面。
                        runner.run_ui_system(ui, world);
                        // 检查模块是否有未应用的修改。
                        let has_mutation = runner.has_mutation();
                        // 在底部右侧创建一个布局区域。
                        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                            // 添加一个“Apply”按钮。只有在有未应用修改时，该按钮才可点击。
                            if ui.add_enabled(has_mutation, Button::new("Apply")).clicked() {
                                // 如果用户点击了按钮，则运行该模块的应用系统来保存更改。
                                runner.run_apply_system(world);
                            }
                        });
                    }
                })
            });
        },
    );
}

/// `SettingsModuleRegistry` 是一个 Bevy 资源，它包含一个从 `Identifier` 到
/// 动态分派的 `ModuleRunner` trait 对象的映射。这允许我们存储和管理所有已注册的设置模块。
#[derive(Resource, Default, Deref)]
pub struct SettingsModuleRegistry(IndexMap<Identifier, Box<dyn ModuleRunner>>);

/// `SettingsModuleDyn` 是 `SettingsModule` 的一个类型擦除的版本。
/// 它持有一个临时的 `storage`，用于在 UI 系统和应用系统之间传递修改后的设置数据。
/// `Storage` 是一个泛型参数，代表了该模块的临时设置状态。
pub struct SettingsModuleDyn<Storage: Send + Sync + 'static> {
    // `Option<Storage>` 用于存储用户在 UI 中所做的、但尚未“应用”的更改。
    // `None` 表示没有更改，`Some` 表示有。
    storage: Option<Storage>,
    // 一个动态分派的只读系统，负责绘制该模块的 UI。
    ui_system: Box<dyn ReadOnlySystem<In = In<(Ui, Option<Storage>)>, Out = Option<Storage>>>,
    // 一个动态分派的系统，负责将 `storage` 中的更改应用到实际的 Bevy 资源中。
    apply_edit_system: Box<dyn System<In = In<Storage>, Out = ()>>,
    // 模块的名称，用于在左侧列表中显示。
    name: Cow<'static, str>,
}

/// `ModuleRunner` 是一个 trait，它为 `SettingsModuleDyn` 提供了一个动态分派的接口。
/// 这使得 `SettingsModuleRegistry` 可以存储不同 `Storage` 类型的 `SettingsModuleDyn` 实例。
pub trait ModuleRunner: Send + Sync + 'static {
    /// 运行 UI 系统来绘制设置界面。
    fn run_ui_system(&mut self, ui: &mut Ui, world: &World);
    /// 运行应用系统来保存更改。
    fn run_apply_system(&mut self, world: &mut World);
    /// 检查是否有未应用的更改。
    fn has_mutation(&self) -> bool;
    /// 返回模块的名称。
    fn name(&self) -> Cow<'static, str>;
}

// 为 `SettingsModuleDyn` 实现 `ModuleRunner` trait。
impl<Storage: Send + Sync + 'static> ModuleRunner for SettingsModuleDyn<Storage> {
    fn run_apply_system(&mut self, world: &mut World) {
        // `take()` 方法会移除 `storage` 中的值，留下 `None`。
        if let Some(storage) = self.storage.take() {
            // 如果存在未应用的更改，则运行应用系统。
            self.apply_edit_system.run(storage, world);
        } else {
            // 这是一个逻辑错误，因为“Apply”按钮在没有更改时不应该可点击。
            error!(
                "Can't apply edit because it hasn't been initialized. system to run: {}",
                self.apply_edit_system.name()
            );
        }
    }
    fn run_ui_system(&mut self, ui: &mut Ui, world: &World) {
        // 创建一个新的子 UI 区域来绘制模块内容。
        let child = ui.new_child(
                UiBuilder::new()
                    .max_rect(ui.max_rect())
                    .layout(*ui.layout())
                    .ui_stack_info(UiStackInfo::default()),
            );
        // 运行只读的 UI 系统。
        // `take()` 将当前的 `storage` 传入系统，系统可能会返回一个新的 `storage`（如果用户修改了设置）。
        self.storage = self
            .ui_system
            .run_readonly((child, self.storage.take()), world);
    }
    fn has_mutation(&self) -> bool {
        // 如果 `storage` 是 `Some`，则表示有未应用的更改。
        self.storage.is_some()
    }
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}
impl<S: Send + Sync + 'static> SettingsModuleDyn<S> {
    /// 从一个具体的 `SettingsModule` 实现创建一个类型擦除的 `SettingsModuleDyn` 实例。
    fn from_module(
        module: impl SettingsModule<SettingsTempStorage = S>,
        world: &mut World,
    ) -> Self {
        Self {
            storage: None, // 初始时没有更改。
            ui_system: module.ui_system(world), // 从模块中获取 UI 系统。
            apply_edit_system: module.apply_edit_system(world), // 从模块中获取应用系统。
            name: module.name(), // 获取模块名称。
        }
    }
}

/// `SettingsModule` trait 定义了一个设置模块必须提供的接口。
pub trait SettingsModule {
    /// `SettingsTempStorage` 是一个关联类型，定义了用于在 UI 和应用逻辑之间传递设置数据的临时结构体。
    type SettingsTempStorage: Send + Sync + 'static;
    /// 返回一个用于绘制 UI 的 Bevy 系统。
    /// 这个系统接收当前的 UI 和可选的临时存储作为输入，并返回一个新的可选临时存储。
    fn ui_system(
        &self,
        world: &mut World,
    ) -> Box<
        dyn ReadOnlySystem<
            In = In<(Ui, Option<Self::SettingsTempStorage>)>,
            Out = Option<Self::SettingsTempStorage>,
        >,
    >;
    /// 返回一个用于应用更改的 Bevy 系统。
    /// 这个系统接收临时存储作为输入，并将其中的值应用到全局的 Bevy 资源中。
    fn apply_edit_system(
        &self,
        world: &mut World,
    ) -> Box<dyn System<In = In<Self::SettingsTempStorage>, Out = ()>>;
    /// 返回模块的显示名称。
    fn name(&self) -> Cow<'static, str>;
}

/// `SettingsModuleStruct` 是 `SettingsModule` trait 的一个便捷实现。
/// 它允许用户通过提供两个闭包（或函数指针）来快速创建一个设置模块，而无需手动实现整个 trait。
pub struct SettingsModuleStruct<Storage, Q, R, M2, M3>
where
    Q: IntoSystem<In<(Ui, Option<Storage>)>, Option<Storage>, M2> + Clone,
    Q::System: ReadOnlySystem,
    R: IntoSystem<In<Storage>, (), M3> + Clone,
    Storage: Send + Sync + 'static,
{
    ui_system: Q,
    apply_edit_system: R,
    name: Cow<'static, str>,
    _phantom: PhantomData<(Storage, M2, M3)>,
}

impl<Storage, Q, R, M2, M3> SettingsModuleStruct<Storage, Q, R, M2, M3>
where
    Q: IntoSystem<In<(Ui, Option<Storage>)>, Option<Storage>, M2> + Clone,
    Q::System: ReadOnlySystem,
    R: IntoSystem<In<Storage>, (), M3> + Clone,
    Storage: Send + Sync,
{
    /// 创建一个新的 `SettingsModuleStruct` 实例。
    pub fn new(ui_system: Q, apply_edit_system: R, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ui_system,
            apply_edit_system,
            name: name.into(),
            _phantom: PhantomData,
        }
    }
}

// 为 `SettingsModuleStruct` 实现 `SettingsModule` trait。
impl<Storage, Q, R, M2, M3> SettingsModule for SettingsModuleStruct<Storage, Q, R, M2, M3>
where
    Q: IntoSystem<In<(Ui, Option<Storage>)>, Option<Storage>, M2> + Clone,
    Q::System: ReadOnlySystem,
    R: IntoSystem<In<Storage>, (), M3> + Clone,
    Storage: Send + Sync + 'static,
{
    type SettingsTempStorage = Storage;
    fn ui_system(
        &self,
        world: &mut World,
    ) -> std::boxed::Box<
        (dyn bevy::prelude::ReadOnlySystem<
            In = In<(egui::Ui, std::option::Option<Storage>)>,
            Out = std::option::Option<Storage>,
        > + 'static),
    > {
        // 将传入的系统（如闭包）转换为一个真正的 Bevy 系统。
        let mut system = IntoSystem::into_system(self.ui_system.clone());
        // 初始化系统，使其可以访问 `World` 中的资源。
        system.initialize(world);
        // 将系统装箱并返回。
        Box::new(system)
    }
    fn apply_edit_system(
        &self,
        world: &mut World,
    ) -> Box<dyn System<Out = (), In = In<Self::SettingsTempStorage>>> {
        let mut system = IntoSystem::into_system(self.apply_edit_system.clone());
        system.initialize(world);
        Box::new(system)
    }
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}
