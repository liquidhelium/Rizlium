use bevy::prelude::*;
use egui::Ui;
use rizlium_chart::editing::{
    chart_path::{LinePath, LinePointPath},
    NotePath,
};
use rizlium_render::GameChart;
use rust_i18n::t;

use crate::{
    settings_module::{SettingsModuleStruct, SettingsRegistrationExt},
    tab_system::TabRegistrationExt,
    EventCollectorResource,
};

use super::editing::ChartEditHistory;

#[derive(Resource, Default)]
pub struct SelectedItem {
    pub item: Option<ChartItem>,
}

pub enum ChartItem {
    LinePoint(LinePointPath),
    Line(LinePath),
    Note(NotePath),
}

pub struct Inspector;

impl Plugin for Inspector {
    fn build(&self, app: &mut App) {
        app.register_tab("inspector", t!("inspector.tab"), logs, || true)
            .init_resource::<SelectedItem>();
    }
}

fn logs(In(ui): In<&mut Ui>, chart: Res<GameChart>, selected: Res<SelectedItem>) {
    let Some(ref item) = selected.item else {
        return;
    };
    match item {
        ChartItem::LinePoint(l) => {}
        _ => (),
    }
}
