use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    DefaultPlugins, window::PresentMode,
};

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::prelude::*;
use rizlium_chart::{__test_chart, chart::{Chart, ChartCache}, VIEW_RECT};
use std::ops::Deref;

macro_rules! return_nothing_change {
    ($($val:ident),+) => {
        if !($(($val.is_changed() || $val.is_added()))||+) {
            return;
        }
    };
}

#[derive(Resource)]
struct GameChart(Chart);
#[derive(Resource)]
struct GameChartCache(ChartCache);
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
struct GameTime(f32);

impl GameChart {
    pub fn get_chart(&self) -> &Chart {
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
        self.beats.value_at(real_time)
    }
}
impl Deref for GameChart {
    type Target = Chart;
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
            // CameraControlPlugin,
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

pub struct CameraControlPlugin;

#[derive(Component)]
pub struct GameCamera;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        // app.add_systems(PreUpdate, update_camera);
    }
}

fn update_camera(chart: Res<GameChart>, time: Res<GameTime>, mut cams: Query<&mut OrthographicProjection, With<GameCamera>>) {
    cams.par_iter_mut().for_each_mut(|mut cam| {
        let scale = chart.cam_scale.value_at(time.0);
        if !scale.is_nan() {
            cam.scale = scale;
        }
        else {
            cam.scale = 0.;
        }
        // todo: still need test
        cam.viewport_origin.x = chart.cam_move.value_at(time.0) / (VIEW_RECT[1][0]- VIEW_RECT[0][0]);
    })
}

mod line_rendering;

fn game_time(chart: Res<GameChart>, time: Res<Time>, mut game_time: ResMut<GameTime>) {
    // todo: start
    let since_start = time.raw_elapsed_wrapped();
    *game_time = GameTime(chart.map_time(since_start.as_secs_f32() - 1.0 /* 1.0 dummy */));
}

fn before_render(mut commands: Commands, mut window: Query<&mut Window>) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                viewport_origin: [0.5, 0.5].into(),
                scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                    width: 900.,
                    height: 1600.,
                },
                ..default()
            },
            transform: Transform {
                translation: [900.,700.0,999.0].into(),
                ..default()
            },
            ..default()
        })
        .insert(GameCamera);
    commands.insert_resource(GameChart(__test_chart()));
    window.single_mut().resolution.set(450., 800.);
    window.single_mut().present_mode = PresentMode::AutoVsync;
}
