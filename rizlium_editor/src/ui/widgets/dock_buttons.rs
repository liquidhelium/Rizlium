use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{RizDockState, RizTabs};

use super::WidgetSystem;

#[derive(SystemParam)]
pub struct DockButtons<'w> {
    tree: ResMut<'w, RizDockState>,
    tabs: Res<'w, RizTabs>,
}

impl WidgetSystem for DockButtons<'static> {
    type Extra<'a> = ();
    fn system(
        world: &mut bevy::prelude::World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _extra: Self::Extra<'_>,
    ) {
        let DockButtons::<'_> { tabs, mut tree } = state.get_mut(world);
        let tree = tree.state.main_surface_mut();
        let opened: Vec<_> = tree.tabs().copied().collect();
        for (i, tab) in tabs.tabs.iter().enumerate() {
            let is_opened = opened.contains(&i);
            if ui.selectable_label(is_opened, tab.name()).clicked() {
                if is_opened {
                    tree.remove_leaf(tree.find_tab(&i).expect("i is opened but then not found?").0);
                    ui.close_menu();
                } else {
                    tree.push_to_first_leaf(i);
                    ui.close_menu();
                }
            }
        }
    }
}
