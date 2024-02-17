use bevy::prelude::*;
use egui::Ui;

use crate::{menu::Custom, tab_system::TabRegistrationExt, widgets::{widget, DockButtons}};

use super::MenuExt;
pub struct Docking;

impl Plugin for Docking {
    fn build(&self, app: &mut App) {
        app.menu_context(|mut ctx| {
            ctx.with_sub_menu("dock_buttons_menu", "Window".into(), 9, |mut sub| {
                sub.add("dock_buttons", "_buttons".into(), Custom(Box::new(|u,w,_| widget::<DockButtons>(w, u))), 0)
            });
        });
        app.register_tab("test.one", "Test", system, ||true);
    }
}
fn system(In(ui): In<&mut Ui>) {
    ui.label("text");
}