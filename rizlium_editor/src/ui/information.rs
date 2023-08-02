use bevy::prelude::World;
use egui::Ui;
use rizlium_render::{GameChart, GameTime};

pub fn information(ui: &mut Ui, world: &mut World) {
    let chart = world.resource::<GameChart>();
    let time = world.resource::<GameTime>();
    ui.code(format!("{:#?}", chart.theme_at(**time)));
}
