// 引入 Bevy 引擎的核心模块。
use bevy::{asset::uuid::Uuid, prelude::*};

// 引入 `bevy_persistent` 插件，用于持久化存储数据。
use bevy_persistent::Persistent;
// 引入 egui 库的组件。
use egui::{Sense, TextEdit, Ui, Widget};
// 引入自定义的 `helium_framework` 框架。
use helium_framework::{
    menu_system::MenuRegistration, // 用于向菜单系统注册项目。
    prelude::{ActionsExt, ToastsStorage}, // 框架的核心预设。
    utils::identifier::Identifier, // 唯一标识符类型。
    widgets::widget, // 用于在 egui 中运行 Bevy 系统的 `widget` 函数。
};
// 引入国际化宏 `t!`。
use rust_i18n::t;

// 引入当前 crate 中的其他模块。
use crate::{
    settings_module::{SettingsModuleStruct, SettingsRegistrationExt}, // 设置模块的注册扩展。
    widgets::dock_button, // 自定义的停靠按钮小部件。
    MainMenuContext, // 主菜单的上下文标识。
    RizTabPresets, // 停靠布局预设的资源类型。
    RizliumDockStateMirror, // 停靠状态的镜像资源。
};
/// `Docking` 是一个 Bevy 插件，用于集成所有与窗口停靠相关的功能。
pub struct Docking;

impl Plugin for Docking {
    fn build(&self, app: &mut App) {
        // 注册一个反射系统，用于在菜单栏中显示停靠按钮。
        // 这个系统是一个闭包，它捕获了上下文类型 `MainMenuContext` 并调用 `dock_button` widget。
        app.reflect_system("docking.button", "A docking Button", |(InMut(ui), InRef(_)): (InMut<Ui>, InRef<MainMenuContext>), world:&mut World| {
            widget(world, ui, dock_button, );
        });
        
        // 在主菜单中注册一个名为 "Window" 的子菜单。
        app.register_submenu::<MainMenuContext>("window",  "Window");
        // 在 "Window" 子菜单下注册一个自定义项 "Docking"，当点击时会执行 "docking.button" 系统。
        app.register_custom::<MainMenuContext>(
            "window/button",
            "Docking",
            "docking.button",
        );
        // 注册一个名为 "docking" 的设置模块。
        app.register_settings_module(
            "docking",
            // 使用 `SettingsModuleStruct` 来快速创建一个设置模块实例。
            SettingsModuleStruct::new(
                docking_ui_module, // 用于渲染 UI 的系统。
                apply_docking_settings, // 用于应用设置更改的系统。
                t!("settings.docking"), // 模块的显示名称（从翻译文件中获取）。
            ),
        );
    }
}
/// `DockSettingState` 是一个临时结构体，用于在设置 UI 中存储正在进行的编辑状态。
/// 它只存在于 `settings_module` 的 UI 系统和应用系统之间。
struct DockSettingState {
    // 如果正在编辑某个预设的名称，则存储该预设的 UUID。
    current_editing_name: Option<Uuid>,
    // 当前选中的预设的 UUID。
    selected_preset: Uuid,
    // 预设列表的临时副本。用户的所有修改都先应用到这个副本上，只有点击“Apply”后才会保存。
    temp_presets: RizTabPresets,
}

// 为 `settings_module` 定义存储类型的别名。
type Storage = DockSettingState;

/// `docking_ui_module` 是一个 Bevy 系统，负责在设置页面中渲染停靠相关的 UI。
fn docking_ui_module(
    In((mut ui, mut state)): In<(Ui, Option<Storage>)>, // 接收 egui UI 和可选的先前状态作为输入。
    presets: Res<Persistent<RizTabPresets>>, // 对持久化的预设资源的只读访问。
    current: Res<RizliumDockStateMirror>, // 对当前停靠状态镜像的只读访问。
) -> Option<Storage> { // 返回 `Some(Storage)` 表示有未应用的更改，`None` 表示没有。
    let current = current.0.as_ref()?; // 如果当前没有停靠状态，则直接返回。
    let mut changed = false; // 标记是否有任何更改发生。
    // 如果这是第一次渲染 UI（`state` 为 `None`），则初始化一个新的状态。
    if state.is_none() {
        state = Some(DockSettingState {
            current_editing_name: None,
            selected_preset: Uuid::nil(), // 初始不选中任何预设。
            temp_presets: presets.clone(), // 从持久化资源中克隆一份预设作为临时副本。
        });
    } else {
        // 如果 `state` 已经是 `Some`，说明这是由用户交互（如编辑名称）触发的重绘，
        // 这本身就意味着状态已更改。
        changed = true;
    }
    let mut state = state.unwrap();
    let mut to_delete_index: Option<usize> = None; // 用于标记待删除的预设索引。
    ui.heading("Docking settings");
    // 显示所有预设的列表。
    egui::ScrollArea::vertical().show(&mut ui, |ui| {
        let current_value = &mut state.selected_preset;
        for (index, (uuid, name, _preset)) in state.temp_presets.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // 如果正在编辑当前预设的名称...
                if state.current_editing_name == Some(*uuid) {
                    // ...则显示一个 `TextEdit` 小部件。
                    let response = TextEdit::singleline(name)
                        .id("current_edit".into())
                        .desired_width(50.0)
                        .ui(ui)
                        .on_hover_text("Click outside to cancel");
                    // 如果 `TextEdit` 失去焦点，则认为编辑完成。
                    if response.lost_focus() {
                        state.current_editing_name = None;
                        changed = true;
                        // 如果名称为空，则重置为默认名称。
                        if name.is_empty() {
                            *name = "Preset".into();
                        }
                    }
                } else if ui.add(egui::Label::new(name.as_str()).sense(Sense::click())).on_hover_text("Double click to edit").double_clicked()
                {
                    // 如果用户双击了标签，则进入名称编辑模式。
                    state.current_editing_name = Some(*uuid);
                    changed = true;
                }
                // 在右侧对齐显示删除按钮和单选按钮。
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Delete").clicked() {
                        to_delete_index = Some(index);
                        changed = true;
                    }
                    // 使用 `radio_value` 来创建单选按钮，用于选择要应用的预设。
                    if ui.radio_value(current_value, *uuid, "").changed() {
                        changed = true;
                    }
                });
            });
        }
    });
    // 添加一个按钮，用于根据当前布局创建一个新的预设。
    if ui.button("Add new preset").clicked() {
        let new_name = format!("Preset {}", state.temp_presets.len() + 1);
        // 从当前布局中移除“设置”标签页本身，因为保存它没有意义。
        let mut current = current.clone();
        current
            .main_surface_mut()
            .retain_tabs(|tab| tab != &mut Identifier::from("settings"));
        state.temp_presets.push((Uuid::new_v4(), new_name, current));
        changed = true;
    }
    // 如果有待删除的预设，则从列表中移除它。
    if let Some(index) = to_delete_index {
        state.temp_presets.remove(index);
        changed = true;
    }
    // 如果发生了任何更改，则返回更新后的状态，否则返回 `None`。
    if changed {
        Some(state)
    } else {
        None
    }
}

/// `apply_docking_settings` 是一个 Bevy 系统，负责将设置 UI 中所做的更改应用到实际的资源中。
fn apply_docking_settings(
    In(storage): In<Storage>, // 接收来自 UI 系统的临时存储作为输入。
    mut current: ResMut<RizliumDockStateMirror>, // 对当前停靠状态镜像的可变访问。
    mut presets: ResMut<Persistent<RizTabPresets>>, // 对持久化预设资源的可变访问。
    mut toast: ResMut<ToastsStorage>, // 用于显示通知消息的资源。
) {
    // 将用户在 UI 中选择的预设应用到当前的停靠状态镜像中。
    if let Some(preset) = storage
        .temp_presets
        .iter()
        .find(|(id, _, _)| *id == storage.selected_preset)
    {
        current.0 = Some(preset.2.clone());
    } else {
        warn!("Selected a non-existing docking preset");
    }
    // 将临时预设列表（可能已被用户修改）保存回持久化存储中。
    if let Err(e) = presets.set(storage.temp_presets) {
        error!("Failed to save docking presets: {}", e);
        toast.error(t!("settings.docking.save_error"));
    } else {
        info!("Docking presets saved successfully");
    }
}
