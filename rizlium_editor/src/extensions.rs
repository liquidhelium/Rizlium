pub mod command_panel;
pub mod debug_flycam;
pub mod docking;
mod editing;
/// 半废弃
mod explorer;
mod game;
pub mod i18n;
mod inspector;
mod project_guide;
pub mod recent;
use self::{
    command_panel::CommandPanel, docking::Docking, editing::Editing, game::Game, i18n::I18nPlugin,
    inspector::Inspector, project_guide::ProjectGuideExtension,
};
use crate::extensions::{debug_flycam::DebugCamExtension, hierarchy_inspector::HierarchyInspector, recent::RecentPlugin};
use bevy::prelude::{App, Plugin};
pub struct ExtensionsPlugin;
impl Plugin for ExtensionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            I18nPlugin,
            Game,
            Docking,
            CommandPanel,
            Editing,
            Inspector,
            DebugCamExtension,
            HierarchyInspector,
            ProjectGuideExtension,
            RecentPlugin
        ));
    }
}
/// Abstract hierarchy for current editing chart
mod hierarchy_inspector;
