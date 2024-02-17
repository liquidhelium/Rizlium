use bevy::prelude::*;
use egui::Ui;

use super::world_view::tools::Tool;

pub fn tool_config(In(ui): In<&mut Ui>, world: &mut World) {
    world.resource_scope(|world, tool: Mut<'_, Tool>| tool.config_ui(ui, world));
}
