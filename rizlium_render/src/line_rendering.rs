use std::borrow::Borrow;

use bevy::ecs::component::Tick;
use bevy::math::vec3a;
use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;
use rizlium_chart::chart::{EasingId, KeyPoint, LinePointData, Tween};

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
pub struct ChartLineId {
    pub(crate) line_idx: usize,
    pub(crate) keypoint_idx: usize,
}

impl ChartLineId {
    pub fn line_idx(&self) -> usize {
        self.line_idx
    }
    
    pub fn keypoint_idx(&self) -> usize {
        self.keypoint_idx
    }
}

#[derive(Component)]
struct LastSyncTick {
    shape: Tick,
    color: Tick,
}

impl LastSyncTick {
    const ZERO: LastSyncTick = LastSyncTick {
        shape: Tick::new(0),
        color: Tick::new(0),
    };
}

#[derive(Bundle)]
pub struct ChartLineBundle {
    layer: RenderLayers,
    line: ChartLine,
    shape: ShapeBundle,
    stoke: Stroke,
    synced_tick: LastSyncTick,
}
impl Default for ChartLineBundle {
    fn default() -> Self {
        Self {
            layer: default(),
            line: default(),
            shape: default(),
            stoke: Stroke::new(Color::NONE, 10.),
            synced_tick: LastSyncTick::ZERO,
        }
    }
}

pub struct ChartLinePlugin;

use super::chart_update;

impl Plugin for ChartLinePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShowLines>()
            .add_systems(
                First,
                add_segments
                    .in_set(LineRenderingSystemSet::SyncChart)
                    .run_if(resource_exists_and_changed::<GameChart>),
            )
            .add_systems(
                PreUpdate,
                associate_segment.run_if(resource_exists_and_changed::<GameChart>),
            )
            .add_systems(
                Update,
                (change_bounding, update_shape, update_color, update_layer)
                    .in_set(LineRenderingSystemSet::Rendering)
                    .run_if(chart_update!()),
            );
    }
}

fn add_segments(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartLine>) {
    let segment_count = chart.segment_count();
    let now_count = lines.iter().count();
    let delta = segment_count - now_count;
    debug!("attempting to add {delta} segments");
    for _ in now_count..segment_count {
        commands.spawn(ChartLineBundle::default());
    }
}

fn change_bounding(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(&mut Aabb, &mut Transform, &Stroke, &ChartLineId)>,
) {
    lines
        .par_iter_mut()
        .for_each(|(mut vis, mut transform, stroke, id)| {
            let line_idx = id.line_idx;
            let keypoint_idx = id.keypoint_idx;
            let (Some(pos1), Some(pos2)) = (
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx, **time),
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx + 1, **time),
            ) else {
                return;
            };
            let extend = Vec2::splat(stroke.options.line_width);
            let pos2: Vec2 = pos2.into();
            let pos1: Vec2 = pos1.into();
            transform.translation = pos1.extend(transform.translation.z);
            let mut rect = Rect::from_corners(Vec2::ZERO, pos2 - pos1);
            rect.min -= extend;
            rect.max += extend;
            *vis = Aabb {
                center: rect.center().extend(0.).into(),
                half_extents: rect.half_size().extend(0.).into(),
            };
        });
}

fn associate_segment(
    mut commands: Commands,
    chart: Res<GameChart>,
    lines: Query<Entity, With<ChartLine>>,
) {
    debug!("running system assocate_segment");
    // return_nothing_change!(chart);
    for (entity, (line_idx, keypoint_idx)) in lines.iter().zip(chart.iter_segment()) {
        commands.entity(entity).insert(ChartLineId {
            line_idx,
            keypoint_idx,
        });
    }
}

// fn update_visual(
//     chart: Res<GameChart>,
//     cache: Res<GameChartCache>,
//     time: Res<GameTime>,
//     mut lines: Query<(&mut Stroke, &mut Path, &ViewVisibility, &ChartLineId)>,
// ) {
//     update_shape(&chart, &cache, &time, &mut lines);
//     update_color(chart, cache, time, lines);
// }

fn update_shape(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(
        &mut Stroke,
        &mut Path,
        &ViewVisibility,
        &ChartLineId,
        &mut LastSyncTick,
        &mut Visibility,
    )>,
) {
    lines
        .par_iter_mut()
        // .batching_strategy(BatchingStrategy::new().batches_per_thread(100))
        .for_each(|(_, mut path, view_vis, id, mut synced, mut vis)| {
            let line_idx = id.line_idx;
            let keypoint_idx = id.keypoint_idx;
            let Some(line) = chart.lines.get(line_idx) else {
                *vis = Visibility::Hidden;
                return;
            };
            let (Some(keypoint1), Some(keypoint2)) = (
                line.points.points().get(keypoint_idx),
                line.points.points().get(keypoint_idx + 1),
            ) else {
                *vis = Visibility::Hidden;
                return;
            };
            if *vis == Visibility::Hidden {
                *vis = Visibility::Visible;
            }
            if !view_vis.get() {
                return;
            }
            if !is_shape_changed(keypoint1, keypoint2)
                && (synced.shape.get() >= chart.last_changed().get())
            {
                return;
            }
            let (Some(pos1), Some(pos2)) = (
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx, **time),
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx + 1, **time),
            ) else {
                return;
            };

            let mut builder = PathBuilder::new();
            builder.move_to(Vec2::ZERO);
            let relative_pos = [pos2[0] - pos1[0], pos2[1] - pos1[1]];
            if !(keypoint1.ease_type == EasingId::Linear
                || pos1[0].approx_eq(&pos2[0])
                || pos1[1].approx_eq(&pos2[1]))
            {
                let mut point_count = ((relative_pos[1]) / 5.).floor();
                if point_count >= 10000. {
                    point_count = 5000.
                }
                builder.reserve(point_count as usize);
                // 0...>1...>2...>3..0'
                (1..point_count as usize)
                    .map(|i| i as f32 / point_count)
                    .map(|t| {
                        [
                            f32::ease(0., relative_pos[0], t, keypoint1.ease_type),
                            <f32 as rizlium_chart::chart::Tween>::lerp(0., relative_pos[1], t),
                        ]
                    })
                    .for_each(|p| {
                        builder.line_to(p.into());
                    });
            }
            builder.line_to(relative_pos.into());
            // connect next segment
            if let Some(pos) =
                chart
                    .with_cache(&cache)
                    .line_pos_at(line_idx, keypoint2.time + 0.01, **time)
            {
                builder.line_to(Vec2::from_array(pos) - Vec2::from_array(pos1));
            }
            *path = builder.build();
            synced.shape = chart.last_changed();
        });
}

const DEBUG_INVISIBLE: Color = Color::LinearRgba(LinearRgba::new(1., 0., 1., 0.2));

fn update_color(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(
        &mut Stroke,
        &mut Path,
        &ViewVisibility,
        &ChartLineId,
        &mut LastSyncTick,
    )>,
) {
    lines
        .par_iter_mut()
        .for_each(|(mut stroke, _, vis, id, mut synced)| {
            if !vis.get() {
                return;
            }
            let line_idx = id.line_idx;
            let keypoint_idx = id.keypoint_idx;
            let Some(line) = chart.lines.get(line_idx) else {
                return;
            };
            let (Some(keypoint1), Some(keypoint2)) = (
                line.points.points().get(keypoint_idx),
                line.points.points().get(keypoint_idx + 1),
            ) else {
                return;
            };
            if !is_shape_changed(keypoint1, keypoint2)
                && (synced.color.get() >= chart.last_changed().get())
            {
                return;
            }
            let (Some(pos1), Some(pos2)) = (
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx, **time),
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx + 1, **time),
            ) else {
                return;
            };
            let pos1: Vec2 = pos1.into();
            let pos2: Vec2 = pos2.into();
            let relative_pos = pos2 - pos1;
            let mut color1 = get_color_of(line, keypoint_idx);
            let mut color2 = get_color_of(line, keypoint_idx + 1);
            if color1.alpha().approx_eq(&0.) && color2.alpha().approx_eq(&0.) {
                color1 = DEBUG_INVISIBLE;
                color2 = DEBUG_INVISIBLE;
            }
            let gradient = LinearGradient {
                start: Vec2::ZERO,
                end: relative_pos,
                stops: vec![GradientStop::new(0., color1), GradientStop::new(1., color2)],
            };
            stroke.brush = Brush::Gradient(gradient.into());
            synced.color = chart.last_changed();
        });
}

fn get_color_of(line: &rizlium_chart::prelude::Line, keypoint_idx: usize) -> Color {
    colorrgba_to_color(
        line.points
            .points()
            .get(keypoint_idx)
            .map(|point| point.relevant.color)
            .unwrap_or_else(|| {
                warn!("point {keypoint_idx} have no color.");
                rizlium_chart::prelude::ColorRGBA {
                    r: 0.,
                    g: 0.,
                    b: 0.,
                    a: 1.,
                }
            }),
    )
}

fn is_shape_changed(
    point1: &KeyPoint<f32, LinePointData>,
    point2: &KeyPoint<f32, LinePointData>,
) -> bool {
    if point1.relevant.canvas == point2.relevant.canvas {
        return false;
    }
    // 一般够用 以后再加复杂情况
    true
}

#[derive(Resource, Default)]
pub struct ShowLines(pub Option<usize>);

fn update_layer(show_lines: Res<ShowLines>, mut lines: Query<(&mut RenderLayers, &ChartLineId)>) {
    // todo: able to only display one line.
    lines.iter_mut().for_each(|(mut layer, line)| {
        if let Some(idx) = show_lines.0 {
            // info!("changing {},{}",line.line_idx, line.keypoint_idx);
            if line.line_idx == idx {
                *layer = RenderLayers::layer(1);
                return;
            }
        }
        *layer = RenderLayers::layer(0);
    });
}
