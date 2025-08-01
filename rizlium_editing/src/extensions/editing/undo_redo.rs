use bevy::ecs::system::ResMut;
use rizlium_render::ChartProvider;
use rizlium_project::ProjectState;

use helium_framework::prelude::ToastsStorage;

use super::ChartEditHistory;

pub fn undo(
    mut history: ResMut<ChartEditHistory>,
    mut chart: ResMut<ProjectState>,
    mut notice: ResMut<ToastsStorage>,
) {
    if let Err(e) = history.undo(chart.chart_mut()) {
        notice.error(e.to_string());
    }
}
pub fn redo(
    mut history: ResMut<ChartEditHistory>,
    mut chart: ResMut<ProjectState>,
    mut notice: ResMut<ToastsStorage>,
) {
    if let Err(e) = history.redo(chart.chart_mut()) {
        notice.error(e.to_string());
    }
}
