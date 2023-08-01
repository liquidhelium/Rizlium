use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
    },
};


use bevy_prototype_lyon::prelude::*;
use rizlium_chart::{
    chart::{Chart},
    VIEW_RECT, prelude::ColorRGBA,
};

use theme::BackgroundThemePlugin;
use time::TimeAndAudioPlugin;
pub use time::TimeManager;

mod line_rendering;
mod time;
mod chart;
mod theme;

use chart::{ChartCachePlugin, GameChartCache};
use chart::GameChart;
use time::GameTime;
#[derive(Resource)]
pub struct GameView(pub Handle<Image>);

pub struct TypeRegisterPlugin;
impl Plugin for TypeRegisterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<line_rendering::ChartLine>()
            .register_type::<GameTime>();
    }
}
pub(crate) fn colorrgba_to_color(color: ColorRGBA) -> Color {
    Color::RgbaLinear {
        red: color.r,
        green: color.g,
        blue: color.b,
        alpha: color.a,
    }
}

pub struct RizliumRenderingPlugin {
    pub config: (),
    pub init_with_chart: Option<Chart>,
}

impl Plugin for RizliumRenderingPlugin {
    fn is_unique(&self) -> bool {
        true
    }
    fn build(&self, app: &mut App) {
        let app = app
            .add_plugins((
                ShapePlugin,
                TypeRegisterPlugin,
                ChartCachePlugin,
                TimeAndAudioPlugin,
                line_rendering::ChartLinePlugin,
                BackgroundThemePlugin
            ))
            .add_systems(Startup, spawn_game_camera)
            .add_systems(Update,bind_gameview);
        if let Some(chart) = self.init_with_chart.clone() {
            app.insert_resource(GameChart::new(chart));
        }
    }
}

fn spawn_game_camera(mut commands: Commands) {
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
                translation: [900., 700.0, 999.0].into(),
                ..default()
            },
            ..default()
        })
        .insert(GameCamera);
}

fn bind_gameview(
    gameview: Option<Res<GameView>>,
    mut game_cameras: Query<&mut Camera, With<GameCamera>>,
) {
    let Some(gameview) = gameview else {
        warn!("No game view exist.");
        return;
    };

    let mut game_camera = game_cameras.single_mut();
    if !matches!(game_camera.target,RenderTarget::Image(_))
    {
        game_camera.target = RenderTarget::Image(gameview.0.clone());
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

fn update_camera(
    chart: Res<GameChart>,
    time: Res<GameTime>,
    mut cams: Query<&mut OrthographicProjection, With<GameCamera>>,
) {
    cams.par_iter_mut().for_each_mut(|mut cam| {
        let scale = chart.cam_scale.value_padding(**time).unwrap();
        if !scale.is_nan() {
            cam.scale = scale;
        } else {
            cam.scale = 0.;
        }
        // todo: still need test
        cam.viewport_origin.x =
            chart.cam_move.value_padding(**time).unwrap() / (VIEW_RECT[1][0] - VIEW_RECT[0][0]);
    })
}






