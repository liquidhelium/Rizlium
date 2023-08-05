

use crate::EditorState;
use bevy::prelude::{Resource, World};
use egui::{Color32, Ui};
use egui_dock::{TabViewer, Tree};

pub mod tab_system;
pub use tab_system::{CachedTab, TabInstace, TabProvider};
pub use tab_system::tabs::*;

macro_rules! tabs {
    ($($tab:path),*) => {
        vec![
            $(Box::new(TabInstace::<$tab>::default()),)*
        ]
    };
}

#[derive(Resource)]
pub struct RizTabs {
    pub tabs: Vec<Box<dyn CachedTab>>,
}

impl Default for RizTabs {
    fn default() -> Self {
        Self {
            tabs: tabs![
                GameViewTab,
                CanvasWindow,
                FileMenu,
                ShowLineControl
            ],
        }
    }
}

pub struct RizTabViewer<'a> {
    pub world: &'a mut World,
    pub editor_state: &'a mut EditorState,
    pub tabs: &'a mut Vec<Box<dyn CachedTab>>,
}

impl TabViewer for RizTabViewer<'_> {
    type Tab = usize;
    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        if let Some(tab) = self.tabs.get_mut(*tab) {
            tab.ui(self.world, ui);
        } else {
            ui.colored_label(
                Color32::LIGHT_RED,
                format!(
                    "UNRESOLVED TAB: tab index {tab} does not exist. There are {} tabs avalible",
                    self.tabs.len()
                ),
            );
        }
    }
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.tabs
            .get(*tab)
            .map(|tab| tab.name())
            .unwrap_or("MISSINGNO".into())
            .into()
    }
}

pub fn dock_window_menu_button(
    ui: &mut Ui,
    text: impl Into<egui::WidgetText>,
    tree: &mut Tree<usize>,
    tabs: &Vec<Box<dyn CachedTab>>,
) {
    let opened: Vec<_> = tree.tabs().copied().collect();
    ui.menu_button(text, |ui| {
        for (i, tab) in tabs.iter().enumerate() {
            let is_opened = opened.contains(&i);
            if ui.selectable_label(is_opened, tab.name()).clicked() {
                if is_opened {
                    tree.remove_leaf(i.into());
                    ui.close_menu();
                } else {
                    tree.push_to_first_leaf(i);
                    ui.close_menu();
                }
            }
        }
    });
}
