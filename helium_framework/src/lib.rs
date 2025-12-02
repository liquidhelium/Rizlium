pub mod hotkeys;
pub mod menu_system;
pub mod notifications;
pub mod reflect_system;
pub mod tab_system;
pub mod utils;
pub mod widgets;

use bevy::app::Plugin;
use hotkeys::HotkeyPlugin;
use menu_system::MenuSystemPlugin;
use notifications::NotificationPlugin;
use reflect_system::ActionPlugin;
use tab_system::TabPlugin;
rizlium_l10n::tl_file!("common" t crate::);

pub struct HeliumFramework;

impl Plugin for HeliumFramework {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins((
            ActionPlugin,
            HotkeyPlugin,
            TabPlugin,
            MenuSystemPlugin,
            NotificationPlugin,
        ));
    }
}

pub mod prelude {
    pub use super::{
        hotkeys::*, menu_system::*, notifications::*, reflect_system::*, tab_system::*, utils::*,
        HeliumFramework,
    };
}
