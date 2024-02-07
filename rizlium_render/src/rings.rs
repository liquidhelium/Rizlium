use bevy::{ecs::query::BatchingStrategy, prelude::*};
use bevy_prototype_lyon::{prelude::*, shapes::Circle};

use crate::{colorrgba_to_color, GameChart, GameChartCache, GameTime};

pub struct RingPlugin;
impl Plugin for RingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            add_rings.run_if(resource_exists_and_changed::<GameChart>()),
        )
        .add_systems(
            Update,
            (rings /*change_ring_color*/,).run_if(chart_update!()),
        );
    }
}

#[derive(Component)]
pub struct Ring(usize);

pub fn rings(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut rings: Query<(&mut Stroke, &mut Transform, &mut Visibility, &Ring)>,
) {
    #[cfg(feature = "trace")]
    let span = info_span!("Ring updates");
    #[cfg(feature = "trace")]
    let _enter = span.enter();
    rings
        .par_iter_mut()
        .batching_strategy(BatchingStrategy::new().batches_per_thread(40))
        .for_each_mut(|(mut stroke, mut transform, mut vis, ring)| {
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
            transform.translation = Vec2::from(pos).extend(20.);
            let Some(line) = chart.lines.get(ring.0)else {
                return;
            };
            let mut color = line.ring_color.value_padding(**time).unwrap();
            if let Some(line_color) = line.line_color.value_padding(**time) {
                color = color + line_color;
            }
            stroke.brush = colorrgba_to_color(color).into();
        });
}

pub fn add_rings(mut commands: Commands, chart: Res<GameChart>, rings: Query<&Ring>) {
    for i in rings.iter().count()..chart.lines.len() {
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::new()
                    .add(&Circle {
                        radius: 50.,
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
            Stroke::new(Color::BLACK, 10.),
            Ring(i),
        ));
    }
}
