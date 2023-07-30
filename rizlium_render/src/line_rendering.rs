use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;
use rizlium_chart::chart::{ColorRGBA, Tween, EasingId};

use bevy_prototype_lyon::prelude::*;

use bevy::{prelude::*, render::view::RenderLayers};

use bevy::render::primitives::Aabb;

use crate::GameChartCache;

use super::{GameChart, GameTime};

use mouse_tracking::prelude::MousePosPlugin;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub struct ChartLine;

#[derive(Component, Reflect, Debug, Default)]
struct ChartLineId {
    line_idx: usize,
    keypoint_idx: usize,
}

#[derive(Bundle)]
pub struct ChartLineBundle {
    layer: RenderLayers,
    line: ChartLine,
    shape: ShapeBundle,
    stoke: Stroke,
}
impl Default for ChartLineBundle {
    fn default() -> Self {
        Self {
            layer: default(),
            line: default(),
            shape: default(),
            stoke: Stroke::new(Color::NONE, 10.),
        }
    }
}

pub struct ChartLinePlugin;

impl Plugin for ChartLinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MousePosPlugin)
            .add_systems(First, (add_lines,))
            .add_systems(PreUpdate, assocate_segment)
            .add_systems(Update, (change_bounding, update_shape, update_color));
    }
}

fn add_lines(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartLine>) {
    return_nothing_change!(chart);
    for _ in lines.iter().count()..chart.get_chart().segment_count() {
        commands.spawn(ChartLineBundle::default());
    }
}

fn change_bounding(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(&mut Aabb, &Stroke, &ChartLineId)>,
) {
    lines.par_iter_mut().for_each_mut(|(mut vis, stroke, id)| {
        let line_idx = id.line_idx;
        let keypoint_idx = id.keypoint_idx;
        let pos1 = chart.with_cache(&cache).pos_for_linepoint_at(line_idx, keypoint_idx, time.0).expect(&format!("{}, {}", line_idx, keypoint_idx));
        let pos2 = chart.with_cache(&cache).pos_for_linepoint_at(line_idx, keypoint_idx+1, time.0).unwrap();
        let extend = Vec2::splat(stroke.options.line_width);
        let mut rect = Rect::from_corners(pos1.into(), pos2.into());
        rect.min -= extend;
        rect.max += extend;
        *vis = Aabb {
            center: rect.center().extend(0.).into(),
            half_extents: rect.half_size().extend(0.).into(),
        };
    });
}

fn assocate_segment(
    mut commands: Commands,
    chart: Res<GameChart>,
    lines: Query<Entity, With<ChartLine>>,
) {
    // return_nothing_change!(chart);
    for (entity, (line_idx, keypoint_idx)) in lines.iter().zip(chart.iter_segment()) {
        commands.entity(entity).insert(ChartLineId {
            line_idx,
            keypoint_idx,
        });
    }
}

fn update_shape(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(&mut Path, &ComputedVisibility, &ChartLineId)>,
) {
    lines.par_iter_mut().for_each_mut(|(mut path, vis, id)| {
        if !vis.is_visible() {
            return;
        }
        let line_idx = id.line_idx;
        let keypoint_idx = id.keypoint_idx;
        let line = &chart.lines[line_idx];
        let keypoint1 = &line.points.points()[keypoint_idx];
        let keypoint2 = &line.points.points()[keypoint_idx+1];

        let mut builder = PathBuilder::new();
        let pos1 = chart.with_cache(&cache).pos_for_linepoint_at(line_idx, keypoint_idx, time.0).unwrap();
        let pos2 = chart.with_cache(&cache).pos_for_linepoint_at(line_idx, keypoint_idx+1, time.0).unwrap();
        builder.move_to(pos1.into());
        // skip straight line
        if !(keypoint1.ease_type == EasingId::Linear || pos1[0].approx_eq(&pos2[0]) || pos1[1].approx_eq(&pos2[1])) {
            let point_count = ((pos2[1] - pos1[1]) / 1.).floor();
            // 0...>1...>2...>3..0'
            (1..point_count as usize)
                .into_iter()
                .map(|i| i as f32 / point_count)
                .map(|t| {
                    [
                        f32::ease(pos1[0], pos2[0], t, keypoint1.ease_type),
                        f32::lerp(pos1[1], pos2[1], t),
                    ]
                })
                .for_each(|p| {
                    builder.line_to(p.into());
                });
        }
        builder.line_to(pos2.into());
        // connect next segment
        if let Some(pos) =
            chart.with_cache(&cache).line_pos_at(line_idx, keypoint2.time+0.01, time.0)
        {
            builder.line_to(pos.into());
        }
        *path = builder.build();
    });
}

const DEBUG_INVISIBLE: Color = Color::rgba_linear(1., 0., 1., 0.2);

fn update_color(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(&mut Stroke, &ComputedVisibility, &ChartLineId)>,
) {
    lines.par_iter_mut().for_each_mut(|(mut stroke, vis, id)| {
        if !vis.is_visible() {
            return;
        }
        let line_idx = id.line_idx;
        let keypoint_idx = id.keypoint_idx;
        let line = &chart.lines[line_idx];
        let pos1 = chart.with_cache(&cache).pos_for_linepoint_at(line_idx, keypoint_idx, time.0).unwrap();
        let pos2 = chart.with_cache(&cache).pos_for_linepoint_at(line_idx, keypoint_idx+1, time.0).unwrap();
        match stroke.brush {
            Brush::Gradient(Gradient::Linear(ref mut gradient)) => {
                gradient.start = pos1.into();
                gradient.end = pos2.into();
            }
            _ => (),
        }

        let mut color1 = colorrgba_to_color(line.point_color.points()[keypoint_idx].value);
        let mut color2 = colorrgba_to_color(line.point_color.points()[keypoint_idx + 1].value);
        if color1.a().approx_eq(&0.) && color2.a().approx_eq(&0.) {
            color1 = DEBUG_INVISIBLE;
            color2 = DEBUG_INVISIBLE;
        }
        let gradient = LinearGradient {
            start: pos1.into(),
            end: pos2.into(),
            stops: vec![
                GradientStop::new(0., color1),
                GradientStop::new(1., color2),
            ],
        };
        stroke.brush = Brush::Gradient(gradient.into());
    });
}

// fn update_layer(
//     chart: Res<GameChart>,
//     mut lines: Query<(&mut RenderLayers, &ChartLineId)>,
// ) {
//     // todo: able to only display one line.
// }

pub(crate) fn colorrgba_to_color(color: ColorRGBA) -> Color {
    Color::RgbaLinear {
        red: color.r ,
        green: color.g,
        blue: color.b ,
        alpha: color.a,
    }
}
