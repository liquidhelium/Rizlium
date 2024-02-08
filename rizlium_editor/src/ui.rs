use crate::{EditorState, RizDockState};
use bevy::prelude::{App, Deref, DerefMut, Resource, World};

use egui::{Color32, RichText, Ui};
use egui_dock::{DockState, TabViewer, Tree};

mod editing;
pub mod tab_system;
pub mod widgets;
pub mod menu;
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
pub struct RizTabPresets(Vec<(String, DockState<usize>)>);

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


pub trait InitRizTabsExt {
    fn init_riztabs(&mut self) -> &mut Self;
}

impl InitRizTabsExt for App {
    fn init_riztabs(&mut self) -> &mut Self {
        let tabs = RizTabs::default();
        self.insert_resource(tabs);
        self
    }
}

pub struct RizTabViewer<'a> {
    pub world: &'a mut World,
    pub editor_state: &'a mut EditorState,
    pub tabs: &'a mut Vec<Box<dyn CachedTab>>,
    pub focused_tab: Option<usize>,
}

impl TabViewer for RizTabViewer<'_> {
    type Tab = usize;
    fn ui(&mut self, ui: &mut egui::Ui, this_index: &mut Self::Tab) {
        if let Some(tab) = self.tabs.get_mut(*this_index) {
            if tab.avaliable(self.world) {
                tab.ui(
                    self.world,
                    ui,
                    self.focused_tab
                        .is_some_and(|focused| focused == *this_index),
                );
            } else {
                ui.colored_label(Color32::GRAY, RichText::new("Not avalible").italics());
            }
        } else {
            ui.colored_label(
                Color32::LIGHT_RED,
                format!(
                    "UNRESOLVED TAB: tab index {this_index} does not exist. There are {} tabs avalible",
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
