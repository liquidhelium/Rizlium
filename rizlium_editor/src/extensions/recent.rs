use bevy::prelude::*;
use egui::Ui;
use helium_framework::{
    prelude::{ActionsExt, MenuRegistration},
    widgets::widget,
};
use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

use crate::{MainMenuContext, project::LoadChartEvent, widgets::recent_file_buttons};

pub struct RecentPlugin;

impl Plugin for RecentPlugin {
    fn build(&self, app: &mut App) {
        app.reflect_system(
            "recent.button",
            "Recent Buttons",
            |(InMut(ui), InRef(_)): (InMut<Ui>, InRef<MainMenuContext>), world: &mut World| {
                widget(world, ui, recent_file_buttons);
            },
        )
        .register_custom::<MainMenuContext>("file/recent", "", "recent.button");
    }
}

#[derive(Resource, Serialize, Deserialize, Debug, Deref, DerefMut)]
pub struct RecentFiles(#[deref] IndexSet<LoadChartEvent>, usize);

impl Default for RecentFiles {
    fn default() -> Self {
        Self(IndexSet::new(), 4)
    }
}

impl RecentFiles {
    pub fn push(&mut self, name: LoadChartEvent) {
        if let (idx, false) = self.0.insert_full(name.clone()) {
            let value = self.0.shift_remove_index(idx).unwrap();
            self.0.insert(value);
        }
        if self.0.len() > self.1 {
            self.0.shift_remove_index(0);
        }
    }
}
