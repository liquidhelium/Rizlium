use bevy::prelude::*;
use egui::Ui;

use crate::widgets::enum_selector;

use super::world_view::tools::Tool;

pub fn tool_config(InMut(ui): InMut<Ui>, world: &mut World) {
    ui.horizontal_wrapped(|ui| {
        ui.label("Current tool:");
        enum_selector(&mut *world.resource_mut::<Tool>(), ui);
    });
    world.resource_scope(|world, tool: Mut<'_, Tool>| tool.config_ui(ui, world));
}
