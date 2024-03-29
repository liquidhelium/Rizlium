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
    mut lines: Query<(&mut Aabb, &Stroke, &ChartLineId)>,
) {
    lines.par_iter_mut().for_each(|(mut vis, stroke, id)| {
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
    mut lines: Query<(&mut Stroke, &mut Path, &ViewVisibility, &ChartLineId)>,
) {
    lines
        .par_iter_mut()
        // .batching_strategy(BatchingStrategy::new().batches_per_thread(100))
        .for_each(|(_, mut path, vis, id)| {
            if !vis.get() {
                if !path.0.as_slice().is_empty() {
                    *path = Path(tess::path::Path::new());
                }
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
                warn!(
                    "Possible wrong segment: line {}, point {}, canvas {}",
                    id.line_idx, id.keypoint_idx, keypoint1.relevant.canvas
                );
            }
            if !(keypoint1.ease_type == EasingId::Linear
                || pos1[0].approx_eq(&pos2[0])
                || pos1[1].approx_eq(&pos2[1]))
            {
                let mut point_count = ((pos2[1] - pos1[1]) / 5.).floor();
                if point_count >= 10000. {
                    point_count = 5000.
                }
                builder.reserve(point_count as usize);
                // 0...>1...>2...>3..0'
                (1..point_count as usize)
                    .map(|i| i as f32 / point_count)
                    .map(|t| {
                        [
                            f32::ease(pos1[0], pos2[0], t, keypoint1.ease_type),
                            <f32 as rizlium_chart::chart::Tween>::lerp(pos1[1], pos2[1], t),
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
    mut lines: Query<(&mut Stroke, &mut Path, &ViewVisibility, &ChartLineId)>,
) {
    lines
        .par_iter_mut()
        .for_each(|(mut stroke, _, vis, id)| {
            if !vis.get() {
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

            if let Brush::Gradient(Gradient::Linear(ref mut gradient)) = stroke.brush {
                gradient.start = pos1.into();
                gradient.end = pos2.into();
            }

            let mut color1 = get_color_of(line, keypoint_idx);
            let mut color2 = get_color_of(line, keypoint_idx + 1);
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

fn get_color_of(line: &rizlium_chart::prelude::Line, keypoint_idx: usize) -> Color {
    colorrgba_to_color(
        line.points
            .points()
            .get(keypoint_idx)
            .map(|point| point.relevant.color)
            .unwrap_or_else(|| {
                warn!("point {keypoint_idx} have no color.");
                rizlium_chart::prelude::ColorRGBA {
                    r:0.,
                    g:0.,
                    b:0.,
                    a:1.,
                }
            }),
    )
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
