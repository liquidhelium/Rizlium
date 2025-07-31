mod dock_buttons;
mod recent_file_buttons;
mod shortcut_display;
pub use dock_buttons::dock_button;
pub use recent_file_buttons::recent_file_buttons;
pub use shortcut_display::shortcut_display;

use bevy::prelude::*;
use egui::Ui;
use std::fmt::Debug;
use strum::IntoEnumIterator;

pub fn enum_selector<T: IntoEnumIterator + Eq + Debug>(value: &mut T, ui: &mut Ui) {
    ui.menu_button(format!("{value:?}"), |ui| {
        for variant in T::iter() {
            let text = format!("{variant:?}");
            if ui.selectable_value(value, variant, text).changed() {
                ui.close_menu();
            };
        }
    });
}
