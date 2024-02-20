use bevy::prelude::*;
use egui::Ui;

use crate::{tab_system::TabRegistrationExt, EventCollectorResource};

use super::editing::ChartEditHistory;

pub struct Inspector;

impl Plugin for Inspector {
    fn build(&self, app: &mut App) {
        app.register_tab("inspector", "Inspector", logs, ||true);
    }
}


fn logs(In(ui): In<&mut Ui>, sub: Res<EventCollectorResource>, edit_history: Res<ChartEditHistory>) {
    // ui.add(egui_tracing::Logs::new(sub.0.clone()));
    for i in edit_history.history_descriptions() {
        ui.label(i.clone());
    }
}