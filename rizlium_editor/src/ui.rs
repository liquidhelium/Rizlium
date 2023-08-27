use crate::EditorState;
use bevy::prelude::{Deref, DerefMut, Resource, World, App};

use egui::{Color32, RichText, Ui};
use egui_dock::{TabViewer, Tree};

mod editing;
pub mod tab_system;
pub mod widgets;
use leafwing_input_manager::dynamic_action::DynAction;
use leafwing_input_manager::prelude::InputMap;
use serde::{Deserialize, Serialize};
pub use tab_system::tabs::*;
pub use tab_system::{CachedTab, TabInstace, TabProvider};

macro_rules! tabs {
    ($($tab:path),*) => {
        vec![
            $(Box::<TabInstace::<$tab>>::default(),)*
        ]
    };
}

#[derive(Resource)]
pub struct RizTabs {
    pub tabs: Vec<Box<dyn CachedTab>>,
}

#[derive(Resource, Serialize, Deserialize, Default, DerefMut, Deref)]
pub struct RizTabPresets(Vec<(String, Tree<usize>)>);

impl Default for RizTabs {
    fn default() -> Self {
        Self {
            tabs: tabs![
                GameViewTab,
                CanvasWindow,
                FileMenu,
                ShowLineControl,
                SplineWindow,
                NoteWindow
            ],
        }
    }
}

impl RizTabs {
    pub fn tab_input_maps(&self) -> InputMap<DynAction> {
        let iter = self
            .tabs
            .iter()
            .map(|tab| tab.hotkeys())
            .flatten()
            .map(|i| (i.0, i.1.to_dyn_action()));
        InputMap::new(iter)
    }
    pub fn register_tab_actions(&self, app: &mut App) {
        for t in &self.tabs {
            t.register_actions(app);
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
            if tab.avaliable(self.world) {
                tab.ui(self.world, ui);
            } else {
                ui.colored_label(Color32::GRAY, RichText::new("Not avalible").italics());
            }
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
            .map_or("MISSINGNO".into(), |tab| tab.name())
            .into()
    }
}

pub fn dock_window_menu_buttons(
    ui: &mut Ui,
    text: impl Into<egui::WidgetText>,
    tree: &mut Tree<usize>,
    tabs: &[Box<dyn CachedTab>],
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
