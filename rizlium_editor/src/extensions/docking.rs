use bevy::prelude::*;

use crate::{
    menu::Custom,
    widgets::{widget, DockButtons},
};

use super::MenuExt;
pub struct Docking;

impl Plugin for Docking {
    fn build(&self, app: &mut App) {
        app.menu_context(|mut ctx| {
            ctx.with_sub_menu("dock_buttons_menu", "Window".into(), 9, |mut sub| {
                sub.add(
                    "dock_buttons",
                    "_buttons".into(),
                    Custom(Box::new(|u, w, _| widget::<DockButtons>(w, u))),
                    0,
                )
            });
        });
    }
}
