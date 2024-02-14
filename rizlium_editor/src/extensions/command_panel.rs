use bevy::{ecs::system::SystemParam, prelude::*};
use egui::{Align2, Layout};

use crate::{
    hotkeys::{Hotkey, HotkeyRegistry, HotkeysExt},
    utils::dot_path::DotPath,
    widgets::WidgetSystem,
    ActionRegistry, ActionsExt,
};

pub struct CommandPanel;

impl Plugin for CommandPanel {
    fn build(&self, app: &mut App) {
        use bevy::input::keyboard::KeyCode::*;
        app.register_action(
            "command_panel.toggle_open",
            "Open or close the command panel",
            toggle_open_command_panel,
        )
        .register_hotkey(
            "command_panel.toggle_open",
            [
                Hotkey::new_global([ControlLeft, P]),
                Hotkey::new([Escape], |r: Res<CommandPanelState>| r.opened),
            ],
        )
        .init_resource::<CommandPanelState>();
    }
}

#[derive(Resource, Default)]
pub struct CommandPanelState {
    opened: bool,
    current_content: String,
}

fn toggle_open_command_panel(mut state: ResMut<CommandPanelState>) {
    state.opened = !state.opened;
    state.current_content.clear();
}

#[derive(SystemParam)]
pub struct CommandPanelImpl<'w> {
    state: ResMut<'w, CommandPanelState>,
    action_storage: Res<'w, ActionRegistry>,
    hotkeys: Res<'w, HotkeyRegistry>,
}

impl WidgetSystem for CommandPanelImpl<'static> {
    type Extra<'a> = ();
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _extra: Self::Extra<'_>,
    ) {
        let ctx = ui.ctx();
        let CommandPanelImpl {
            mut state,
            action_storage,
            hotkeys,
        } = state.get_mut(world);
        if !state.opened {
            return;
        }
        let mut ready_to_run: Option<DotPath> = None;
        let mut panel_rect = ctx.screen_rect().shrink(20.);
        panel_rect.set_height(20.);
        panel_rect.set_width(400.0f32.min(panel_rect.width()));
        egui::Area::new("commands")
            .movable(false)
            .order(egui::Order::Foreground)
            .anchor(Align2::CENTER_TOP, [0., panel_rect.top()])
            .show(ctx, |ui| {
                set_menu_style(ui.style_mut());
                egui::Frame::popup(ui.style()).show(ui, |ui| {
                    ui.set_max_width(panel_rect.width());
                    ui.set_max_height(ctx.screen_rect().height() / 2.);
                    ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                        ui.add_sized(
                            panel_rect.size(),
                            egui::TextEdit::singleline(&mut state.current_content),
                        );
                        egui::ScrollArea::new([false, true])
                            // .max_height(ctx.screen_rect().height() / 2.)
                            .max_width(panel_rect.width())
                            .auto_shrink([false, true])
                            .show(ui, |ui| {
                                action_storage.iter().for_each(|(id, action)| {
                                    let mut button = egui::Button::new(
                                        id.to_string()
                                            + " "
                                            + action
                                                .input_type_info()
                                                .type_path_table()
                                                .short_path()
                                            + "\n"
                                            + action.get_description(),
                                    );
                                    if let Some(hotkey) = hotkeys.get(id) {
                                        if !hotkey.is_empty() {
                                            let text = hotkey
                                                .iter()
                                                .map(Hotkey::hotkey_text)
                                                .collect::<Vec<_>>()
                                                .join(" or ");
                                            button = button.shortcut_text(text);
                                        }
                                    }
                                    if ui.add(button).clicked_by(egui::PointerButton::Primary) {
                                        ready_to_run = Some(id.clone())
                                    }
                                })
                            });
                    })
                });
            });
        if let Some(ready) = ready_to_run {
            world.resource_scope(|world, mut action_storage: Mut<'_, ActionRegistry>| {
                if let Err(e) = action_storage.run_instant(&ready, (), world) {
                    error!("Error executing {ready}, {e}");
                }
                if world.resource::<CommandPanelState>().opened {
                    world.resource_mut::<CommandPanelState>().opened = false;
                }
            });
        }
    }
}

fn set_menu_style(style: &mut egui::Style) {
    style.spacing.button_padding = [2.0, 2.0].into();
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
}
