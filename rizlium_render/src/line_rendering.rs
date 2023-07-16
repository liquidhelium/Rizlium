use rizlium_chart::chart::ColorRGBA;

use bevy_prototype_lyon::prelude::*;

use bevy::prelude::*;

use bevy::render::primitives::Aabb;


use super::{GameChart, GameTime};

use mouse_tracking::prelude::MousePosPlugin;

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
pub(crate) struct ChartLine;

pub struct ChartLinePlugin;

impl Plugin for ChartLinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MousePosPlugin)
            .add_systems(PreUpdate, (add_lines, change_bounding))
            .add_systems(Update, (update_shape, update_color));
    }
}

pub(crate) fn add_lines(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartLine>) {
    for _ in lines.iter().count()..chart.get_chart().segment_count() {
        commands.spawn((
            ShapeBundle::default(),
            Fill::brush(Color::NONE),
            Stroke::new(Color::BLACK, 10.0),
            ChartLine,
        ));
    }
}

pub(crate) fn change_bounding(chart: Res<GameChart>, time: Res<GameTime> ,mut lines: Query<(&mut Aabb, &Stroke), With<ChartLine>>) {
    for ((mut vis, stroke), (line_idx, keypoint_idx)) in lines.iter_mut().zip(chart.iter_segment())
    {
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
    }
}

pub(crate) fn update_shape(
    chart: Res<GameChart>, time: Res<GameTime>,
    mut lines: Query<(&mut Path, &ComputedVisibility), With<ChartLine>>,
) {
    for ((mut path, vis), (line_idx, keypoint_idx)) in lines.iter_mut().zip(chart.iter_segment()) {
        if !vis.is_visible() {
            continue;
        }
        let line = &chart.lines[line_idx];
        let range =
            line.points.points[keypoint_idx].time + 0.01..line.points.points[keypoint_idx + 1].time;
        let iter = std::iter::successors(Some(range.start), move |a| {
            range.contains(a).then_some(a + 0.01)
        })
        .map(|this_time| line.try_pos_at_time(this_time, /*game_time*/ time.0))
        .flatten();

        let mut builder = PathBuilder::new();
        builder.move_to(line.pos_for(keypoint_idx, time.0).unwrap().into());
        iter.for_each(|point| {
            builder.line_to(point.into());
        });
        builder.line_to(line.pos_for(keypoint_idx + 1, time.0).unwrap().into());
        // connect next segment
        if let Some(pos) =
            line.try_pos_at_time(line.points.points[keypoint_idx + 1].time + 0.1, time.0)
        {
            builder.line_to(pos.into());
        }
        *path = builder.build();
    }
}

pub(crate) fn update_color(chart: Res<GameChart>, time: Res<GameTime>,mut lines: Query<(&mut Stroke, &ComputedVisibility), With<ChartLine>>) {
    for ((mut stroke,vis), (line_index, keypoint_index)) in lines.iter_mut().zip(chart.iter_segment()) {
        if !vis.is_visible() {
            continue;
        }
        let line = &chart.lines[line_index];
        let pos1 = line.pos_for(keypoint_index, time.0).unwrap();
        let pos2 = line.pos_for(keypoint_index + 1, time.0).unwrap();
        match stroke.brush {
            Brush::Gradient(Gradient::Linear(ref mut gradient)) => {
                gradient.start = pos1.into();
                gradient.end = pos2.into();
            },
            _ => ()
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
    }
}

pub(crate) fn colorrgba_to_color(color: ColorRGBA) -> Color {
    Color::RgbaLinear {
        red: color.r / 255.0,
        green: color.g / 255.0,
        blue: color.b / 255.0,
        alpha: color.a / 255.0,
    }
}
