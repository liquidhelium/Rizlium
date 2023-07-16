use bevy::{prelude::*, DefaultPlugins, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin}};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use mouse_tracking::{
    prelude::InitWorldTracking,
    MainCamera,
};
use rizlium_chart::{
    __test_chart,
    chart::RizChart,
};
use std::ops::Deref;

#[derive(Resource)]
struct GameChart(RizChart);
#[derive(Resource,Reflect, Default)]
#[reflect(Resource)]
struct GameTime(f32);

impl GameChart {
    pub fn get_chart(&self) -> &RizChart {
        &self.0
    }
    pub fn iter_segment(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .enumerate()
            .map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points.len() - 1))
            .flatten()
    }
    pub fn map_time(&self, real_time: f32) -> f32 {
        //todo
        real_time
    }
}
impl Deref for GameChart {
    type Target = RizChart;
    fn deref(&self) -> &Self::Target {
        self.get_chart()
    }
}



pub fn start() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(GameTime(0.))
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            ShapePlugin,
            TypeRegisterPlugin,
            line_rendering::ChartLinePlugin,
            FrameTimeDiagnosticsPlugin::default(),
            LogDiagnosticsPlugin::default(),
        ))
        .add_systems(Startup, before_render)
        .add_systems(PreUpdate, game_time)
        .run();
}
pub struct TypeRegisterPlugin;
impl Plugin for TypeRegisterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<line_rendering::ChartLine>()
            .register_type::<GameTime>();
    }
}

mod line_rendering;

fn game_time(chart: Res<GameChart>, time: Res<Time>, mut game_time: ResMut<GameTime>) {
    // todo: start
    let since_start = time.raw_elapsed_wrapped();
    game_time.0 = chart.map_time(since_start.as_secs_f32()-1.0 /* 1.0 dummy */);
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
