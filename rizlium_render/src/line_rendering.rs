use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;
use rizlium_chart::chart::{EasingId, Tween};

use bevy_prototype_lyon::prelude::*;

use bevy::{prelude::*, render::view::RenderLayers};

use bevy::render::primitives::Aabb;

use crate::GameChartCache;

use super::{colorrgba_to_color, GameChart, GameTime};

#[derive(Debug, PartialEq, Eq, SystemSet, Clone, Hash)]
pub enum LineRenderingSystemSet {
    SyncChart,
    Rendering,
}

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

use super::chart_update;

impl Plugin for ChartLinePlugin {
    fn build(&self, app: &mut App) {
        app
        .init_resource::<ShowLines>()
        .add_systems(
            First,
            (add_lines, assocate_segment)
                .in_set(LineRenderingSystemSet::SyncChart)
                .run_if(chart_update!()),
        )
        .add_systems(
            Update,
            (change_bounding, update_shape, update_color, update_layer)
                .in_set(LineRenderingSystemSet::Rendering)
                .run_if(chart_update!()),
        );
    }
}

fn add_lines(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartLine>) {
    for _ in lines.iter().count()..chart.segment_count() {
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
        let pos1 = chart
            .with_cache(&cache)
            .pos_for_linepoint_at(line_idx, keypoint_idx, **time)
            .expect("Get pos for line point failed");
        let pos2 = chart
            .with_cache(&cache)
            .pos_for_linepoint_at(line_idx, keypoint_idx + 1, **time)
            .unwrap();
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
        let keypoint2 = &line.points.points()[keypoint_idx + 1];

        let mut builder = PathBuilder::new();
        let pos1 = chart
            .with_cache(&cache)
            .pos_for_linepoint_at(line_idx, keypoint_idx, **time)
            .unwrap();
        let pos2 = chart
            .with_cache(&cache)
            .pos_for_linepoint_at(line_idx, keypoint_idx + 1, **time)
            .unwrap();
        builder.move_to(pos1.into());
        if pos1[1].approx_eq(&0.) && pos2[1].approx_eq(&0.) {
            warn!("Possible wrong segment: line {}, point {}, canvas {}", id.line_idx, id.keypoint_idx, keypoint1.relevent);
        }
        // k = 1600意味着1个像素内就经过了一个屏幕
        let k =( pos2[1] - pos1[1])/ (pos2[0] - pos1[0]);
        // skip straight line
        if !(keypoint1.ease_type == EasingId::Linear
            || pos1[0].approx_eq(&pos2[0])
            || pos1[1].approx_eq(&pos2[1]) || k.abs() >= 1400.0)
        {
            let point_count = ((pos2[1] - pos1[1]) / 1.).floor();
            if point_count > 10000. {
                warn!("long segment found, line = {}, point = {} (k= {k})", id.line_idx, id.keypoint_idx);
            }
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
            chart
                .with_cache(&cache)
                .line_pos_at(line_idx, keypoint2.time + 0.01, **time)
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
        let pos1 = chart
            .with_cache(&cache)
            .pos_for_linepoint_at(line_idx, keypoint_idx, **time)
            .unwrap();
        let pos2 = chart
            .with_cache(&cache)
            .pos_for_linepoint_at(line_idx, keypoint_idx + 1, **time)
            .unwrap();
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
            stops: vec![GradientStop::new(0., color1), GradientStop::new(1., color2)],
        };
        stroke.brush = Brush::Gradient(gradient.into());
    });
}

#[derive(Resource, Default)]
pub struct ShowLines(pub Option<usize>);

fn update_layer(show_lines: Res<ShowLines>, mut lines: Query<(&mut RenderLayers, &ChartLineId)>) {
    // todo: able to only display one line.
    lines.for_each_mut(|(mut layer, line)| {
        if let Some(idx) = show_lines.0 {
            // info!("changing {},{}",line.line_idx, line.keypoint_idx);
            if line.line_idx == idx {
                *layer = RenderLayers::layer(1);
                return;
            }
        }
        *layer = RenderLayers::layer(0);
    })
}