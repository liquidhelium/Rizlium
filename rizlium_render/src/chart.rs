use bevy::prelude::*;
use rizlium_chart::prelude::*;
use std::ops::Deref;
#[derive(Resource)]
pub struct GameChart(Chart);

impl GameChart {
    pub fn new(chart: Chart) -> Self {
        Self(chart)
    }
    pub fn iter_segment(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .enumerate()
            .map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points().len() - 1))
            .flatten()
    }
    pub fn iter_note(&self) -> impl Iterator<Item = (usize, usize)> + '_  {
        self.lines
            .iter()
            .enumerate()
            .map(|(i, l)| std::iter::repeat(i).zip(0..l.notes.len()))
            .flatten()
    }
}

#[derive(Resource, Default)]
pub struct GameChartCache(pub ChartCache);

impl Deref for GameChartCache {
    type Target = ChartCache;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ChartCachePlugin;

impl Plugin for ChartCachePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            chart_cache.run_if(resource_exists_and_changed::<GameChart>()),
        );
    }
}
impl Deref for GameChart {
    type Target = Chart;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
fn chart_cache(
    mut commands: Commands,
    chart: Res<GameChart>,
    cache: Option<ResMut<GameChartCache>>,
) {
    let Some(mut cache) = cache else {
        info!("add cache");
        commands.insert_resource(GameChartCache(ChartCache ::from_chart(&chart)));
        return;
    };
    info!("update cache");
    cache.0.update_from_chart(&chart);
}
