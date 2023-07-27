use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
    DefaultPlugins,
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
#[derive(Resource, Default)]
struct GameChartCache(ChartCache);

impl Deref for GameChartCache {
    type Target = ChartCache;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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
            .map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points().len() - 1))
            .flatten()
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
        .init_resource::<GameChartCache>()
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
        .add_systems(First, chart_cache)
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
    fn build(&self, _app: &mut App) {
        // app.add_systems(PreUpdate, update_camera);
    }
}

fn update_camera(chart: Res<GameChart>, time: Res<GameTime>, mut cams: Query<&mut OrthographicProjection, With<GameCamera>>) {
    cams.par_iter_mut().for_each_mut(|mut cam| {
        let scale = chart.cam_scale.value_padding(time.0).unwrap();
        if !scale.is_nan() {
            cam.scale = scale;
        }
        else {
            cam.scale = 0.;
        }
        // todo: still need test
        cam.viewport_origin.x = chart.cam_move.value_padding(time.0).unwrap() / (VIEW_RECT[1][0]- VIEW_RECT[0][0]);
    })
}

mod line_rendering;

fn chart_cache(chart: Res<GameChart>, mut cache: ResMut<GameChartCache>) {
    return_nothing_change!(chart);
    info!("update cache");
    cache.0.update_from_chart(&chart);
}

fn game_time(cache: Res<GameChartCache>,time: Res<Time>, mut game_time: ResMut<GameTime>) {
    // todo: start
    let since_start = time.raw_elapsed_seconds();
    *game_time = GameTime(cache.0.beat.value_padding(since_start).unwrap());
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
}
