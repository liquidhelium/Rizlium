use bevy::ecs::system::ResMut;
use rizlium_render::GameChart;

use super::ChartEditHistory;

pub fn undo(mut history: ResMut<ChartEditHistory>, mut chart: ResMut<GameChart>) {
    history.undo(&mut chart);
}
pub fn redo(mut history: ResMut<ChartEditHistory>, mut chart: ResMut<GameChart>) {
    history.redo(&mut chart);
}