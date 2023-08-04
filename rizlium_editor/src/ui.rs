use std::fmt::Display;

use crate::EditorState;
use bevy::prelude::World;
use egui::Ui;
use egui_dock::{TabViewer, Tree};
use strum::{EnumIter, IntoEnumIterator};

use self::{dummy_window::dummy_window, game_view::GameViewTab, widget_system::WidgetId};

mod dummy_window;
mod game_view;
mod information;
mod widget_system;
mod file_menu;
pub use widget_system::{widget, WidgetSystem};

#[derive(Debug, PartialEq, Eq, EnumIter, Clone, Copy)]
pub enum RizliumTab {
    GameView,
    Information,
    Dummy2,
    Dummy3,
}

impl Display for RizliumTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GameView => f.write_str("Game view"),
            Self::Information => f.write_str("Information"),
            _ => f.write_str("Dummy <N>"),
        }
    }
}

pub struct RizTabViewer<'a> {
    pub world: &'a mut World,
    pub editor_state: &'a mut EditorState,
}

impl TabViewer for RizTabViewer<'_> {
    type Tab = RizliumTab;
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            RizliumTab::GameView => widget::<GameViewTab>(self.world, ui, WidgetId::new("1")),
            RizliumTab::Information => information::information(ui, self.world),
            RizliumTab::Dummy2 => widget::<file_menu::FileMenu>(self.world, ui, WidgetId::new("2").into()),
            _ => dummy_window(ui),
        }
    }
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.to_string().into()
    }
}

pub fn dock_window_menu_button(
    ui: &mut Ui,
    text: impl Into<egui::WidgetText>,
    tree: &mut Tree<RizliumTab>,
) {
    let opened: Vec<_> = tree.tabs().copied().enumerate().collect();
    ui.menu_button(text, |ui| {
        for i in RizliumTab::iter() {
            let value = opened.iter().find(|(_, tab)| i == *tab).map(|a| a.0);
            let contains = value.is_some();
            if ui.selectable_label(contains, i.to_string()).clicked() {
                if contains {
                    tree.remove_leaf((value.unwrap()).into());
                    ui.close_menu();
                } else {
                    tree.push_to_focused_leaf(i);
                    ui.close_menu();
                }
            }
        }
    });
}
