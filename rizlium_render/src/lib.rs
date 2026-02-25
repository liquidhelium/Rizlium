#![allow(clippy::type_complexity)]
use std::marker::PhantomData;

use bevy::{
    prelude::*,
    camera::{RenderTarget, visibility::RenderLayers},
    core_pipeline::oit::OrderIndependentTransparencySettings,
};
// use bevy::core_pipeline::fxaa::Fxaa;

use bevy_hanabi::HanabiPlugin;
use bevy_prototype_lyon::prelude::*;
use masks::MaskPlugin;
use notes::ChartNotePlugin;
use rings::RingPlugin;
use rizlium_chart::prelude::ColorRGBA;

pub use masks::MASK_LAYER;
pub use rizlium_chart;
use theme::BackgroundThemePlugin;

// 长类型让我抓狂
#[macro_export]
macro_rules! chart_update {
    ($provider:ty) => {
    {
        use crate::GameChartCache;
        P::has_chart_system().and(resource_changed::<P>.or(resource_changed::<GameTime>)).and(resource_exists::<GameChartCache>)
    }
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

pub struct RizliumRenderingPlugin<P: ChartProvider> {
    pub config: (),
    pub manual_time_control: bool,
    _marker: std::marker::PhantomData<P>,
}

impl<P: ChartProvider> Default for RizliumRenderingPlugin<P> {
    fn default() -> Self {
        Self {
            config: (),
            manual_time_control: false,
            _marker: std::marker::PhantomData,
        }
    }
}
#[macro_export]
macro_rules! default_ph {
    ($typ:ty) => {
        impl<P: ChartProvider> Default for $typ {
            fn default() -> Self {
                Self(std::marker::PhantomData)
            }
        }
    };
}

impl<P: ChartProvider> Plugin for RizliumRenderingPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ShapePlugin,
            TypeRegisterPlugin,
            HanabiPlugin,
            ChartCachePlugin::<P>::default(),
            line_rendering::ChartLinePlugin::<P>::default(),
            TimeAndAudioPlugin,
            BackgroundThemePlugin::<P>::default(),
            ChartNotePlugin::<P>::default(),
            RingPlugin::<P>::default(),
            MaskPlugin::<P>::default(),
            // HitParticlePlugin::<P>::default(),
            CameraControlPlugin::<P>::default(),
        ))
        .add_systems(Startup, spawn_game_camera)
        .add_systems(PostUpdate, bind_gameview);
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
                scaling_mode: bevy::camera::ScalingMode::Fixed {
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
            RenderTarget::None { size: (10,10).into() }
            // Fxaa::default(),
        ))
        .insert(GameCamera);
}

// TODO: don't run continuously
fn bind_gameview(
    gameview: Option<Res<GameView>>,
    mut game_cameras: Query<&mut RenderTarget, With<GameCamera>>,
) {
    let Some(gameview) = gameview else {
        warn!("No game view exist.");
        return;
    };

    let Ok(mut render_target) = game_cameras.single_mut() else {
        warn!("No game camera found.");
        return;
    };
    if !matches!(*render_target, RenderTarget::Image(_)) {
        *render_target = RenderTarget::Image(gameview.0.clone().into());
    }
}

pub struct CameraControlPlugin<P>(PhantomData<P>);

impl<P> Default for CameraControlPlugin<P> {
    fn default() -> Self {
        Self(std::marker::PhantomData)
    }
}

#[derive(Component)]
pub struct GameCamera;

impl<P: ChartProvider> Plugin for CameraControlPlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_systems(PreUpdate, update_camera::<P>.run_if(chart_update!(P)));
    }
}

fn update_camera<P: ChartProvider>(
    time: Res<GameTime>,
    mut query: Query<(&mut Projection, &mut Transform), With<GameCamera>>,
    chart: Res<P>,
    cache: Res<GameChartCache>,
) -> Result<()> {
    let chart = chart.chart(); // ensure loaded
    let cam_move = chart.cam_move.value_padding(time.0).unwrap_or_else(|| {
        warn!("Camera movement padding failed");
        0.0
    });
    let cam_scale = chart
        .cam_scale
        .value_padding(time.0)
        .into_iter()
        .flat_map(|m| {
            // prevent zero scale, or nan will break everything
            if m.abs() < f32::EPSILON {
                None
            } else {
                Some(m)
            }
        })
        .next()
        .unwrap_or_else(|| {
            warn!("Camera scale padding failed");
            1.0
        });
    let (mut proj, mut transform) = query.single_mut()?;
    if let Projection::Orthographic(ortho) = &mut *proj {
        ortho.scale = 1. / cam_scale;
    }
    transform.translation.x = cam_move;
    Ok(())
}
