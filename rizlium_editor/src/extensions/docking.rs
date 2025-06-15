use bevy::prelude::*;

use helium_framework::{
    menu::{Custom, MenuExt},
    widgets::{dock_button, widget},
};
pub struct Docking;

impl Plugin for Docking {
    fn build(&self, app: &mut App) {
        app.menu_context(|mut ctx| {
            ctx.with_sub_menu("dock_buttons_menu", "Window".into(), 9, |mut sub| {
                sub.add(
                    "dock_buttons",
                    "_buttons".into(),
                    Custom(Box::new(|u, w, _| widget(w, u, dock_button))),
                    0,
                )
            });
        });
    }
}
