use bevy::prelude::{Deref, DerefMut, Resource, World};

use egui::Ui;
use egui_dock::{DockState, TabViewer};

pub mod widgets;
use helium_framework::prelude::{TabId, TabRegistry};
use serde::{Deserialize, Serialize};


#[derive(Resource, Serialize, Deserialize, Default, DerefMut, Deref)]
pub struct RizTabPresets(Vec<(String, DockState<TabId>)>);

pub struct RizTabViewer<'a> {
    pub world: &'a mut World,
    pub registry: &'a mut TabRegistry,
}

impl<'a> TabViewer for RizTabViewer<'a> {
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
