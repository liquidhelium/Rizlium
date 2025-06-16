use bevy::ecs::system::ResMut;
use rizlium_render::GameChart;

use helium_framework::prelude::ToastsStorage;

use super::ChartEditHistory;

pub fn undo(
    mut history: ResMut<ChartEditHistory>,
    mut chart: ResMut<GameChart>,
    mut notice: ResMut<ToastsStorage>,
) {
    if let Err(e) = history.undo(&mut chart) {
        notice.error(e.to_string());
    }
}
pub fn redo(
    mut history: ResMut<ChartEditHistory>,
    mut chart: ResMut<GameChart>,
    mut notice: ResMut<ToastsStorage>,
) {
    if let Err(e) = history.redo(&mut chart) {
        notice.error(e.to_string());
    }
}
