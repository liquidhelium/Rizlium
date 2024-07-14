use bevy::prelude::{Deref, DerefMut, World, Resource};

use egui::Ui;
use egui_dock::{DockState, TabViewer};

pub mod menu;
pub mod tab_system;
pub mod widgets;
use serde::{Deserialize, Serialize};

use self::tab_system::{TabId, TabRegistry};

#[derive(Resource, Serialize, Deserialize, Default, DerefMut, Deref)]
pub struct RizTabPresets(Vec<(String, DockState<TabId>)>);

pub struct RizTabViewerNext<'a> {
    pub world: &'a mut World,
    pub registry: &'a mut TabRegistry,
}

impl<'a> TabViewer for RizTabViewerNext<'a> {
    type Tab = TabId;
    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        self.registry
            .get(tab)
            .map(|t| t.title())
            .unwrap_or("MISSINGNO".into())
            .into()
    }
    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        self.registry.tab_ui(ui, self.world, tab);
    }
}
