use bevy::prelude::*;
use egui::Ui;
use rust_i18n::t;

use crate::{settings_module::{SettingsModuleStruct, SettingsRegistrationExt}, tab_system::TabRegistrationExt, EventCollectorResource};

use super::editing::ChartEditHistory;

pub struct Inspector;

impl Plugin for Inspector {
    fn build(&self, app: &mut App) {
        app.register_tab("inspector", t!("inspector.tab"), logs, ||true)
            .register_settings_module("inspector", SettingsModuleStruct::new(settings_ui, settings_apply, "Inspector"));
    }
}


fn logs(In(ui): In<&mut Ui>, sub: Res<EventCollectorResource>, edit_history: Res<ChartEditHistory>) {
    // ui.add(egui_tracing::Logs::new(sub.0.clone()));
    for i in edit_history.history_descriptions() {
        ui.label(i.clone());
    }
}

fn settings_ui(In((mut ui, edit)): In<(Ui, Option<()>)>) -> Option<()> {
    let ui = &mut ui;
    ui.button("text").clicked().then_some(()).or(edit)
}

fn settings_apply(In(()): In<()>) {
    debug!("applied!");
}