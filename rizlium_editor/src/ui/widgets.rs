mod dock_buttons;
mod recent_file_buttons;
mod shortcut_display;
pub use dock_buttons::dock_button;
pub use recent_file_buttons::recent_file_buttons;
pub use shortcut_display::shortcut_display;

use bevy::prelude::*;
use egui::{Response, Ui};
use std::fmt::Debug;
use strum::IntoEnumIterator;

pub fn enum_selector<T: IntoEnumIterator + Eq + Debug>(value: &mut T, ui: &mut Ui) -> Response {
    let response = ui.menu_button(format!("{value:?}"), |ui| {
        for variant in T::iter() {
            let text = format!("{variant:?}");
            let selectable_value = ui.selectable_value(value, variant, text);
            if selectable_value.changed() {
                ui.close();
                return Some(selectable_value);
            };
        }
        None
    });
    response.inner.flatten().unwrap_or(response.response)
}
