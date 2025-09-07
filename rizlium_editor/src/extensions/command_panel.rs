// 引入 Bevy 引擎的核心预设模块。
use bevy::prelude::*;
// 引入 egui 库的 `Align2` 和 `Layout`，用于 UI 布局。
use egui::{Align2, Layout};

// 引入自定义的 `helium_framework` 框架的核心组件和工具。
use helium_framework::{prelude::*, utils::identifier::Identifier};

/// `CommandPanel` 是一个 Bevy 插件，用于设置命令面板功能。
/// 命令面板是一个弹出式窗口，允许用户搜索并执行所有已注册的动作（actions）。
pub struct CommandPanel;

impl Plugin for CommandPanel {
    fn build(&self, app: &mut App) {
        // 使用 `use` 语句将 `KeyCode` 枚举成员引入当前作用域，以简化代码。
        use bevy::input::keyboard::KeyCode::*;
        // 注册一个名为 "command_panel.toggle_open" 的反射系统。
        // 这个系统可以在其他地方通过其字符串 ID 来调用。
        app.reflect_system(
            "command_panel.toggle_open", // 系统的唯一 ID。
            "Show all commands", // 对用户的简短描述。
            toggle_open_command_panel, // 要注册的函数。
        )
        // 为这个动作注册快捷键。
        .register_hotkey(
            "command_panel.toggle_open",
            [
                // 注册一个全局快捷键 `Ctrl + P`。
                Hotkey::new_global([ControlLeft, KeyP]),
                // 注册一个条件性快捷键 `Escape`。
                // 这个快捷键只有在 `|r: Res<CommandPanelState>| r.opened` 这个条件为真时才会触发，
                // 也就是说，只有当命令面板已经打开时，按 `Escape` 键才会关闭它。
                Hotkey::new([Escape], |r: Res<CommandPanelState>| r.opened),
            ],
        )
        // 初始化 `CommandPanelState` 资源，用于存储命令面板的状态。
        .init_resource::<CommandPanelState>();
    }
}

/// `CommandPanelState` 是一个 Bevy 资源，用于存储命令面板的当前状态。
#[derive(Resource, Default)]
pub struct CommandPanelState {
    opened: bool, // 面板是否打开。
    current_content: String, // 搜索框中的当前文本内容。
}

/// `toggle_open_command_panel` 是一个 Bevy 系统，用于切换命令面板的打开/关闭状态。
fn toggle_open_command_panel(mut state: ResMut<CommandPanelState>) {
    // 切换 `opened` 字段的布尔值。
    state.opened = !state.opened;
    // 每次打开/关闭时，清空搜索框的内容。
    state.current_content.clear();
}

/// `command_panel` 是一个 Bevy `widget` 系统，负责渲染命令面板的 UI。
/// 它通过 `In` 参数接收一个对 `egui::Ui` 的可变引用。
pub fn command_panel(
    In(ui): In<&mut egui::Ui>,
    mut state: ResMut<CommandPanelState>, // 命令面板的状态。
    action_storage: Res<RSystemRegistry>, // 存储所有已注册动作的注册表。
    hotkeys: Res<HotkeyRegistry>, // 存储所有已注册快捷键的注册表。
    mut action: Actions, // `Actions` 是一个系统参数，用于执行动作。
) {
    let ctx = ui.ctx(); // 获取 egui 的上下文。
    // 如果面板是关闭的，则直接返回，不渲染任何东西。
    if !state.opened {
        return;
    }
    // `ready_to_run` 用于暂存用户点击的动作 ID。
    // 我们不能在渲染循环中直接执行动作，因为这会可变地借用 `World`，与 UI 渲染冲突。
    // 所以我们先记录下来，在渲染结束后再执行。
    let mut ready_to_run: Option<Identifier> = None;
    // 计算面板的初始矩形区域。
    let mut panel_rect = ctx.screen_rect().shrink(20.);
    panel_rect.set_height(20.);
    panel_rect.set_width(400.0f32.min(panel_rect.width()));
    // 使用 `egui::Area` 创建一个浮动窗口。
    egui::Area::new("commands".into())
        .movable(false) // 不允许用户拖动。
        .order(egui::Order::Foreground) // 确保它渲染在其他 UI 之上。
        .anchor(Align2::CENTER_TOP, [0., panel_rect.top()]) // 将其锚定在屏幕顶部中央。
        .show(ctx, |ui| {
            // 应用自定义的菜单样式。
            set_menu_style(ui.style_mut());
            // 使用 `egui::Frame::popup` 来创建一个带边框和阴影的弹出式框架。
            egui::Frame::popup(ui.style()).show(ui, |ui| {
                ui.set_max_width(panel_rect.width());
                ui.set_max_height(ctx.screen_rect().height() / 2.);
                // 使用垂直布局。
                ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    // 添加一个单行文本输入框作为搜索栏。
                    ui.add_sized(
                        panel_rect.size(),
                        egui::TextEdit::singleline(&mut state.current_content),
                    );
                    // 创建一个可滚动的区域来显示命令列表。
                    egui::ScrollArea::new([false, true])
                        .max_width(panel_rect.width())
                        .auto_shrink([false, true])
                        .show(ui, |ui| {
                            // 遍历所有已注册的动作。
                            action_storage.iter().for_each(|(id, action)| {
                                // 为每个动作创建一个按钮。
                                // TODO: 这里应该根据 `state.current_content` 来过滤显示的动作。
                                let mut button = egui::Button::new(
                                    id.to_string()
                                        + " "
                                        + &action.input
                                        + "\n"
                                        + action.description.as_str(),
                                );
                                // 检查该动作是否有对应的快捷键。
                                if let Some(hotkey) = hotkeys.get(id) {
                                    if !hotkey.is_empty() {
                                        // 将快捷键格式化并显示在按钮的右侧。
                                        let text = hotkey
                                            .iter()
                                            .map(Hotkey::hotkey_text)
                                            .collect::<Vec<_>>()
                                            .join(" or ");
                                        button = button.shortcut_text(text);
                                    }
                                }
                                // 如果按钮被点击，则记录下其 ID，准备执行。
                                if ui.add(button).clicked_by(egui::PointerButton::Primary) {
                                    ready_to_run = Some(id.clone())
                                }
                            })
                        });
                })
            });
        });
    // 在 UI 渲染结束后，检查是否有待执行的动作。
    if let Some(ready) = ready_to_run {
        // 执行动作。
        if let Err(e) = action.run_action(&ready, ()) {
            error!("Error executing {ready}, {e}");
        }
        // 执行动作后，关闭命令面板。
        if state.opened {
            state.opened = false;
        }
    }
}

/// `set_menu_style` 是一个辅助函数，用于修改 egui 的样式，使其看起来更像一个菜单。
fn set_menu_style(style: &mut egui::Style) {
    style.spacing.button_padding = [2.0, 2.0].into();
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
}
