use bevy::prelude::*;
use egui::Ui;

use crate::{tab_system::TabRegistrationExt, EventCollectorResource};

pub struct Inspector;

impl Plugin for Inspector {
    fn build(&self, app: &mut App) {
        app.register_tab("inspector".into(), "Inspector", logs, ||true);
    }
}


fn logs(In(ui): In<&mut Ui>, sub: Res<EventCollectorResource>) {
    ui.add(egui_tracing::Logs::new(sub.0.clone()));
}