use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

use crate::{RizDockState, RizTabs};

use super::WidgetSystem;

#[derive(SystemParam)]
pub struct DockButtons<'w> {
    state: ResMut<'w, RizDockState>,
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
        let DockButtons::<'_> { tabs, mut state } = state.get_mut(world);
        let state = &mut state.state;
        let opened:Vec<_> = state.iter_all_tabs().map(|(_, t)| t).copied().collect();
        for (i, tab) in tabs.tabs.iter().enumerate() {
            let is_opened = opened.contains(&i);
            if ui.selectable_label(is_opened, tab.name()).clicked() {
                if is_opened {
                    state.remove_tab(state.find_tab(&i).expect("i is opened but then not found?"));
                    ui.close_menu();
                } else {
                    state.add_window(vec![i]);
                    ui.close_menu();
                }
            }
        }
    }
}
