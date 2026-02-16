use bevy::prelude::*;
use bevy_persistent::Persistent;

use crate::{editor_actions::EditorCommands, RecentFiles};

pub fn recent_file_buttons(
    In(ui): In<&mut egui::Ui>,
    recent: Res<Persistent<RecentFiles>>,
    mut editor_commands: EditorCommands,
) {
    for entry in recent.get().iter().rev() {
        if ui.button(entry).clicked() {
            editor_commands.load_chart(entry.clone());
            ui.close();
        }
    }
}
