use bevy::{ecs::system::SystemParam, prelude::*};
use egui::{Align2, Color32, Layout};

use crate::{
    hotkeys::{HotkeyListener, HotkeysExt},
    widgets::WidgetSystem,
    ActionsExt,
};

pub struct CommandPanel;

impl Plugin for CommandPanel {
    fn build(&self, app: &mut App) {
        use bevy::input::keyboard::KeyCode::*;
        app.register_action("command_panel.toggle_open", toggle_open_command_panel)
            .register_hotkey(HotkeyListener::new_global(
                "command_panel.toggle_open",
                [ControlLeft, P],
            ))
            .init_resource::<CommandPanelState>();
    }
}

#[derive(Resource, Default)]
pub struct CommandPanelState {
    opened: bool,
    current_content: String,
}

fn toggle_open_command_panel(mut state: ResMut<CommandPanelState>) {
    info!("command panel: {}", state.opened);
    state.opened = !state.opened;
    state.current_content.clear();
}

#[derive(SystemParam)]
pub struct CommandPanelImpl<'w> {
    state: ResMut<'w, CommandPanelState>,
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
        let CommandPanelImpl { mut state } = state.get_mut(world);
        if !state.opened {
            return;
        }
        let mut panel_rect = ctx.screen_rect().shrink(20.);
        panel_rect.set_height(20.);
        panel_rect.set_width(400.0f32.min(panel_rect.width()));
        ctx.debug_painter().debug_rect(panel_rect, Color32::DEBUG_COLOR, "commands");
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
                            .auto_shrink(false)
                            .show(ui, |ui| {
                                for _ in 0..500 {
                                    // ui.menu_button("title", |ui| {
                                    ui.button("Text\ntest");
                                    // });
                                }
                            });
                    })
                });
            });
    }
}


fn set_menu_style(style: &mut egui::Style) {
    style.spacing.button_padding = [2.0, 0.0].into();
    style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
    style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
    style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
}