use bevy_prototype_lyon::{shapes::Circle, prelude::*};
use bevy::prelude::*;

use crate::{GameChart, GameChartCache, GameTime};

#[derive(Component)]
pub(crate) struct Ring(usize);

pub(crate) fn rings(chart: Res<GameChart>,cache: Res<GameChartCache>, time: Res<GameTime>, mut rings: Query<(&mut Transform, &mut Visibility, &Ring)>) {
    rings.par_iter_mut().for_each_mut(|(mut transform,mut vis,ring)| {
        let chart_with_cache = chart.with_cache(&cache);
        let Some(pos) = chart_with_cache.line_pos_at(ring.0, **time, **time) else {
            *vis = Visibility::Hidden;
            return;
        };
        *vis = Visibility::Visible;
        *transform = transform.with_translation(Vec2::from(pos).extend(0.));
    })
}

pub(crate) fn add_rings(mut commands: Commands, chart: Res<GameChart>, rings: Query<&Ring>) {
    for i in rings.iter().count()..chart.lines.len() {
        commands.spawn(
            (ShapeBundle {
                path: GeometryBuilder::new().add(&Circle {
                    radius: 50.,
                    center: [0.,0.].into()
                }).build(),
                transform: Transform::from_translation(Vec3 { x: 0., y: 0., z: 10. }),
                ..default()
            },
            Stroke::new(Color::BLACK, 10.),
            Ring(i))
        );
    }
}
