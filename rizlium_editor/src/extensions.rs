pub mod command_panel;
pub mod debug_flycam;
pub mod docking;
mod editing;
mod game;
pub mod i18n;
mod inspector;
pub mod explorer;


use bevy::prelude::{App, Plugin};


use crate::extensions::debug_flycam::DebugCamExtension;

use self::{
    command_panel::CommandPanel, docking::Docking, editing::Editing, game::Game, i18n::I18nPlugin,
    inspector::Inspector,
};

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
        ));
    }
}