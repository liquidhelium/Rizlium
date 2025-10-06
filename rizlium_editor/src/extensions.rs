pub mod command_panel;
pub mod debug_flycam;
pub mod docking;
mod editing;
mod game;
pub mod i18n;
mod inspector;

pub mod explorer;
mod project_guide;
use self::{
    command_panel::CommandPanel, docking::Docking, editing::Editing, game::Game, i18n::I18nPlugin,
    inspector::Inspector, project_guide::ProjectGuideExtension,
};
use crate::extensions::debug_flycam::DebugCamExtension;
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
            ProjectGuideExtension,
        ));
    }
}
