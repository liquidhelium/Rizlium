use bevy::{prelude::*, render::primitives::Aabb, DefaultPlugins};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::{
    entity::SimpleShapeBundle,
    brush::{Brush, GradientStop, LinearGradient},
    prelude::*,
};
use mouse_tracking::{
    prelude::{InitMouseTracking, InitWorldTracking, MousePosPlugin},
    MainCamera, MousePosWorld,
};
use rizlium_chart::{
    __test_chart,
    chart::{ColorRGBA, Line, RizChart},
};
use std::ops::Deref;

#[derive(Resource)]
struct GameChart(RizChart);

impl GameChart {
    pub fn get_chart(&self) -> &RizChart {
        &self.0
    }
    pub fn iter_segment(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .enumerate()
            .map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points.len()-1))
            .flatten()
    }
}
impl Deref for GameChart {
    type Target = RizChart;
    fn deref(&self) -> &Self::Target {
        self.get_chart()
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct ChartLine;

pub fn start() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            ShapePlugin,
            TypeRegisterPlugin,
            ChartLinePlugin,
        ))
        .add_systems(Startup, before_render)
        .run();
}
pub struct TypeRegisterPlugin;
impl Plugin for TypeRegisterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChartLine>();
    }
}

pub struct ChartLinePlugin;
impl Plugin for ChartLinePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MousePosPlugin)
            .add_systems(PreUpdate, (add_lines,change_bounding))
            .add_systems(Update, (update_shape, update_color));
    }
}
fn add_lines(mut commands: Commands, chart: Res<GameChart>, lines: Query<&ChartLine>) {
    for _ in lines.iter().count()..chart.get_chart().segment_count() {
        commands.spawn((
            ShapeBundle::default(),
            Fill::brush(Color::NONE),
            Stroke::new(Color::BLACK, 10.0),
            ChartLine,
        ));
    }
}

fn change_bounding(chart: Res<GameChart>, mut lines: Query<(&mut Aabb,&Stroke), With<ChartLine>>) {
    for ((mut vis, stroke), (line_idx, keypoint_idx)) in lines.iter_mut().zip(chart.iter_segment()) {
        let line =&chart.lines[line_idx];
        let pos1 = line.pos_for(keypoint_idx, 0.0).unwrap();
        let pos2 = line.pos_for(keypoint_idx+1, 0.0).unwrap();
        let extend = Vec2::splat(stroke.options.line_width);
        let mut rect = Rect::from_corners(pos1.into(), pos2.into());
        rect.min -= extend;
        rect.max += extend;
        *vis = Aabb {
            center: rect.center().extend(0.).into(),
            half_extents: rect.half_size().extend(0.).into()
        };
    }
}


fn update_shape(chart: Res<GameChart>, mut lines: Query<&mut Path, With<ChartLine>>) {
    for (mut path, (line_idx, keypoint_idx)) in lines.iter_mut().zip(chart.iter_segment()) {
        let line = &chart.lines[line_idx];
        let range =
            line.points.points[keypoint_idx].time + 0.01..line.points.points[keypoint_idx + 1].time;
        let iter = std::iter::successors(Some(range.start), move |a| {
            range.contains(a).then_some(a + 0.01)
        })
        .map(|time| line.try_pos_at_time(time, /*game_time*/ 0.0))
        .flatten();

        let mut builder = PathBuilder::new();
        builder.move_to(line.pos_for(keypoint_idx, 0.0).unwrap().into());
        iter.for_each(|point| {
            builder.line_to(point.into());
        });
        builder.line_to(line.pos_for(keypoint_idx + 1, 0.0).unwrap().into());
        if let Some(pos) =
            line.try_pos_at_time(line.points.points[keypoint_idx + 1].time + 0.1, 0.0)
        {
            builder.line_to(pos.into());
        }
        *path = builder.build();
    }
}
fn update_color(chart: Res<GameChart>, mut lines: Query<&mut Stroke, With<ChartLine>>) {
    for (mut stroke, (line_index, keypoint_index)) in lines.iter_mut().zip(chart.iter_segment()) {
            let line = &chart.lines[line_index];

            let pos1 = line.pos_for(keypoint_index, 0.0).unwrap();
            let pos2 = line.pos_for(keypoint_index + 1, 0.0).unwrap();
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


fn colorrgba_to_color(color: ColorRGBA) -> Color {
    Color::RgbaLinear {
        red: color.r / 255.0,
        green: color.g / 255.0,
        blue: color.b / 255.0,
        alpha: color.a / 255.0,
    }
}

fn before_render(mut commands: Commands, mut window: Query<&mut Window>) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                viewport_origin: [-0.5, 0.].into(),
                scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                    width: 900.,
                    height: 1600.,
                },
                ..default()
            },
            ..default()
        })
        .add(InitWorldTracking)
        .insert(MainCamera);
    commands.insert_resource(GameChart(__test_chart()));
    window.single_mut().resolution.set(450., 800.);
}
