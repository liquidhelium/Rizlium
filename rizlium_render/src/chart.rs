use bevy::prelude::*;
use rizlium_chart::prelude::*;

use crate::default_ph;

/// Trait for types that can provide chart data as a Bevy resource
pub trait ChartProvider: Resource {
    fn has_chart_system() -> impl Condition<()>;

    /// Get the current chart
    fn chart(&self) -> &Chart;

    /// Get mutable access to the chart
    fn chart_mut(&mut self) -> &mut Chart;

    /// Iterate over all segments in the chart
    fn iter_segment(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.chart()
            .lines
            .iter()
            .enumerate()
            .flat_map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points().len() - 1))
    }
    /// Iterate over all points in the chart
    fn iter_point(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.chart()
            .lines
            .iter()
            .enumerate()
            .flat_map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points().len()))
    }

    /// Iterate over all notes in the chart
    fn iter_note(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.chart()
            .lines
            .iter()
            .enumerate()
            .flat_map(|(i, l)| std::iter::repeat(i).zip(0..l.notes.len()))
    }
}

#[derive(Resource, Default, Deref)]
pub struct GameChartCache(pub ChartCache);

pub struct ChartCachePlugin<P: ChartProvider>(std::marker::PhantomData<P>);

default_ph!(ChartCachePlugin<P>);

impl<P: ChartProvider> Plugin for ChartCachePlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            chart_cache::<P>.run_if(P::has_chart_system().and(resource_changed::<P>)),
        );
    }
}

fn chart_cache<P: ChartProvider>(
    mut commands: Commands,
    provider: Res<P>,
    cache: Option<ResMut<GameChartCache>>,
) {
    let Some(mut cache) = cache else {
        info!("add cache");
        commands.insert_resource(GameChartCache(ChartCache::from_chart(provider.chart())));
        return;
    };
    info!("update cache");
    cache.0.update_from_chart(provider.chart());
}
