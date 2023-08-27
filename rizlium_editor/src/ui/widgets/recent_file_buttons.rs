use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::{EditorCommands, RecentFiles};

use super::WidgetSystem;

#[derive(SystemParam)]
pub struct RecentButtons<'w, 's> {
    recent: Res<'w, Persistent<RecentFiles>>,
    editor_commands: EditorCommands<'s>,
}

impl WidgetSystem for RecentButtons<'static, 'static> {
    type Extra<'a> = ();
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _extra: Self::Extra<'_>,
    ) {
        let RecentButtons { recent , mut editor_commands} = state.get(world);
        for entry in recent.get().iter() {
            if ui.button(entry).clicked() {
                editor_commands.load_chart(entry.clone());
                ui.close_menu();
            }
        }
    }
}
