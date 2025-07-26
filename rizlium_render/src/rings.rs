use bevy::{ecs::batching::BatchingStrategy, prelude::*};
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::{colorrgba_to_color, ChartProvider, GameChartCache, GameTime};

pub const RING_Z: f32 = 20.;
pub struct RingPlugin<P: ChartProvider>(std::marker::PhantomData<P>);

default_ph!(RingPlugin<P>);

impl<P: ChartProvider> Plugin for RingPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            add_rings::<P>.run_if(P::has_chart_system().and(resource_changed::<P>)),
        )
        .add_systems(
            Update,
            (rings::<P> /*change_ring_color*/,).run_if(chart_update!(P)),
        );
    }
}

#[derive(Component)]
pub struct Ring(usize);

fn rings<P: ChartProvider>(
    provider: Res<P>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut rings: Query<(&mut Stroke, &mut Transform, &mut Visibility, &Ring)>,
) {
    #[cfg(feature = "trace")]
    let span = info_span!("Ring updates");
    #[cfg(feature = "trace")]
    let _enter = span.enter();
    let chart = provider.chart();
    rings
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().batches_per_thread(40))
        .for_each(|(mut stroke, mut transform, mut vis, ring)| {
            #[cfg(feature = "trace")]
            let span = info_span!("single ring");
            #[cfg(feature = "trace")]
            let _enter = span.enter();
            let chart_with_cache = chart.with_cache(&cache);
            let Some(pos) = chart_with_cache.line_pos_at(ring.0, **time, **time) else {
                if *vis != Visibility::Hidden {
                    *vis = Visibility::Hidden;
                }
                return;
            };
            if *vis != Visibility::Visible {
                *vis = Visibility::Visible;
            }
            transform.translation = Vec2::from(pos).extend(RING_Z);
            let Some(line) = chart.lines.get(ring.0) else {
                return;
            };
            let color = line.ring_color.value_padding(**time).unwrap_or_default();
            stroke.brush = colorrgba_to_color(color).into();
        });
}

fn add_rings<P: ChartProvider>(mut commands: Commands, chart: Res<P>, rings: Query<&Ring>) {
    for i in rings.iter().count()..chart.chart().lines.len() {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::new()
                    .add(&Circle {
                        radius: 43.,
                        center: [0., 0.].into(),
                    })
                    .build(),
                transform: Transform::from_translation(Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 10.,
                }),
                ..default()
            },
            Stroke::new(Color::BLACK, 8.),
            Ring(i),
        ));
    }
}
