use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::{EditorCommands, RecentFiles};

use super::WidgetSystem;

#[derive(SystemParam)]
pub struct RecentButtons<'w> {
    recent: Res<'w, Persistent<RecentFiles>>,
}

impl WidgetSystem for RecentButtons<'static> {
    type Extra<'a> = &'a mut EditorCommands;
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        commands: Self::Extra<'_>,
    ) {
        let RecentButtons { recent } = state.get(&world);
        for entry in recent.get().iter() {
            if ui.button(entry).clicked() {
                commands.load_chart(entry.clone());
                ui.close_menu();
            }
        }
    }
}
