use bevy::{
    core_pipeline::{fxaa::Fxaa, oit::OrderIndependentTransparencySettings},
    prelude::*,
    render::{camera::RenderTarget, view::RenderLayers},
};

use bevy_hanabi::HanabiPlugin;
use bevy_prototype_lyon::prelude::*;
use masks::MaskPlugin;
use notes::ChartNotePlugin;
use rings::RingPlugin;
use rizlium_chart::{chart::Chart, prelude::ColorRGBA};

pub use masks::MASK_LAYER;
pub use rizlium_chart;
use theme::BackgroundThemePlugin;
pub use time_and_audio::TimeManager;

// 长类型让我抓狂
#[macro_export]
macro_rules! chart_update {
    () => {
        resource_exists::<GameChart>
            // .and(resource_exists_and_changed::<GameChart>.or(resource_changed::<GameTime>))
    };
}

mod chart;
mod line_rendering;
pub use line_rendering::{ChartLine, ChartLineId, ShowLines};
mod hit_parcticles;
mod theme;
mod time_and_audio;

pub mod notes;

pub use chart::*;
pub use time_and_audio::*;

use crate::hit_parcticles::HitParticlePlugin;
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
    Color::Srgba(Srgba {
        red: color.r,
        green: color.g,
        blue: color.b,
        alpha: color.a,
    })
}

pub struct RizliumRenderingPlugin {
    pub config: (),
    pub init_with_chart: Option<Chart>,
    pub manual_time_control: bool,
}

impl Plugin for RizliumRenderingPlugin {
    fn build(&self, app: &mut App) {
        let app = app
            .add_plugins((
                ShapePlugin,
                TypeRegisterPlugin,
                HanabiPlugin,
                ChartCachePlugin,
                line_rendering::ChartLinePlugin,
                TimeAndAudioPlugin {
                    manual_time_control: self.manual_time_control,
                },
                BackgroundThemePlugin,
                ChartNotePlugin,
                RingPlugin,
                MaskPlugin,
                HitParticlePlugin,
            ))
            .add_systems(Startup, spawn_game_camera)
            .add_systems(PostUpdate, bind_gameview);
        if let Some(chart) = self.init_with_chart.clone() {
            app.insert_resource(GameChart::new(chart));
        }
    }
}

mod masks;

mod rings;

fn spawn_game_camera(mut commands: Commands) {
    commands
        .spawn((
            Camera2d,
            Projection::Orthographic(OrthographicProjection {
                viewport_origin: [0.5, masks::RING_OFFSET].into(),
                scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                    width: 900.,
                    height: 1600.,
                },
                ..OrthographicProjection::default_2d()
            }),
            Transform {
                translation: [0., 0., 999.0].into(),
                ..default()
            },
            OrderIndependentTransparencySettings::default(),
            RenderLayers::from_layers(&[MASK_LAYER, 0]),
            Msaa::Off,
            Fxaa::default(),
        ))
        .insert(GameCamera);
}

// TODO: don't run continuously
fn bind_gameview(
    gameview: Option<Res<GameView>>,
    mut game_cameras: Query<&mut Camera, With<GameCamera>>,
) {
    let Some(gameview) = gameview else {
        warn!("No game view exist.");
        return;
    };

    let Ok(mut game_camera) = game_cameras.single_mut() else {
        warn!("No game camera found.");
        return;
    };
    use bevy::render::camera::ImageRenderTarget;
    if !matches!(game_camera.target, RenderTarget::Image(_)) {
        game_camera.target = RenderTarget::Image(ImageRenderTarget::from(gameview.0.clone()));
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
