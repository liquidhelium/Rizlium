use bevy::prelude::*;
use rizlium_chart::prelude::*;
#[derive(Resource, Deref, DerefMut)]
pub struct GameChart(Chart); // TODO gate edit history behind this, so that invalid edit won't appear

impl GameChart {
    pub fn new(chart: Chart) -> Self {
        Self(chart)
    }
    pub fn iter_segment(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .enumerate()
            .flat_map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points().len() - 1))
    }
    pub fn iter_note(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .enumerate()
            .flat_map(|(i, l)| std::iter::repeat(i).zip(0..l.notes.len()))
    }
}

#[derive(Resource, Default, Deref)]
pub struct GameChartCache(pub ChartCache);

pub struct ChartCachePlugin;

impl Plugin for ChartCachePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            chart_cache.run_if(resource_exists_and_changed::<GameChart>),
        );
    }
}
fn chart_cache(
    mut commands: Commands,
    chart: Res<GameChart>,
    cache: Option<ResMut<GameChartCache>>,
) {
    let Some(mut cache) = cache else {
        info!("add cache");
        commands.insert_resource(GameChartCache(ChartCache::from_chart(&chart)));
        return;
    };
    info!("update cache");
    cache.0.update_from_chart(&chart);
}
