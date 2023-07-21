use bevy_prototype_lyon::prelude::tess::geom::euclid::approxeq::ApproxEq;
use rizlium_chart::chart::ColorRGBA;

use bevy_prototype_lyon::prelude::*;

use bevy::{prelude::*, render::view::RenderLayers};

use bevy::render::primitives::Aabb;

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
    time: Res<GameTime>,
    mut lines: Query<(&mut Aabb, &Stroke, &ChartLineId)>,
) {
    lines.par_iter_mut().for_each_mut(|(mut vis, stroke, id)| {
        let line_idx = id.line_idx;
        let keypoint_idx = id.keypoint_idx;
        let line = &chart.lines[line_idx];
        let pos1 = line.pos_for(keypoint_idx, time.0).unwrap();
        let pos2 = line.pos_for(keypoint_idx + 1, time.0).unwrap();
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
        let keypoint = &line.points.points[keypoint_idx];

        let mut builder = PathBuilder::new();
        let pos1 = line.pos_for(keypoint_idx, time.0).unwrap();
        let pos2 = line.pos_for(keypoint_idx + 1, time.0).unwrap();
        builder.move_to(pos1.into());
        // straight line
        if !(keypoint.ease == 0 || pos1[0].approx_eq(&pos2[0]) || pos1[1].approx_eq(&pos2[1])) {
            let range = line.points.points[keypoint_idx].time + 0.01
                ..line.points.points[keypoint_idx + 1].time;
            let iter = std::iter::successors(Some(range.start), move |a| {
                range.contains(a).then_some(a + 0.01)
            })
            .map(|this_time| line.try_pos_at_time(this_time, /*game_time*/ time.0))
            .flatten();
            iter.for_each(|point| {
                builder.line_to(point.into());
            });
        }
        builder.line_to(pos2.into());
        // connect next segment
        if let Some(pos) =
            line.try_pos_at_time(line.points.points[keypoint_idx + 1].time + 0.1, time.0)
        {
            builder.line_to(pos.into());
        }
        *path = builder.build();
    });
}

fn update_color(
    chart: Res<GameChart>,
    time: Res<GameTime>,
    mut lines: Query<(&mut Stroke, &ComputedVisibility, &ChartLineId)>,
) {
    lines.par_iter_mut().for_each_mut(|(mut stroke, vis, id)| {
        if !vis.is_visible() {
            return;
        }
        let line_index = id.line_idx;
        let keypoint_index = id.keypoint_idx;
        let line = &chart.lines[line_index];
        let pos1 = line.pos_for(keypoint_index, time.0).unwrap();
        let pos2 = line.pos_for(keypoint_index + 1, time.0).unwrap();
        match stroke.brush {
            Brush::Gradient(Gradient::Linear(ref mut gradient)) => {
                gradient.start = pos1.into();
                gradient.end = pos2.into();
            }
            _ => (),
        }

        let color1 = line.point_color.points[keypoint_index].value;
        let color2 = line.point_color.points[keypoint_index + 1].value;
        let gradient = LinearGradient {
            start: pos1.into(),
            end: pos2.into(),
            stops: vec![
                GradientStop::new(0., colorrgba_to_color(color1)),
                GradientStop::new(1., colorrgba_to_color(color2)),
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
        red: color.r / 255.0,
        green: color.g / 255.0,
        blue: color.b / 255.0,
        alpha: color.a / 255.0,
    }
}
