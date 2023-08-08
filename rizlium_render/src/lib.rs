use bevy::{
    prelude::*,
    render::{camera::RenderTarget, primitives::Aabb},
};

use bevy_prototype_lyon::{prelude::*, shapes::Rectangle};
use chart_loader::ChartLoadingPlugin;
use notes::ChartNotePlugin;
use rings::RingPlugin;
use rizlium_chart::{chart::Chart, prelude::ColorRGBA, VIEW_RECT};

use theme::BackgroundThemePlugin;
use time_and_audio::TimeAndAudioPlugin;
pub use time_and_audio::TimeManager;

// 长类型让我抓狂
#[macro_export]
macro_rules! chart_update {
    () => {
        resource_exists::<GameChart>().and_then(
            resource_exists_and_changed::<GameChart>().or_else(resource_changed::<GameTime>()),
        )
    };
}

mod chart;
mod line_rendering;
pub use line_rendering::ShowLines;
mod chart_loader;
mod theme;
mod time_and_audio;
pub use chart_loader::LoadChartEvent;

mod notes;

pub use chart::*;
pub use time_and_audio::*;
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
                BackgroundThemePlugin,
                ChartLoadingPlugin,
                ChartNotePlugin,
                RingPlugin,
            ))
            .add_systems(Startup, (spawn_game_camera, init_mask))
            .add_systems(PostUpdate, (bind_gameview, update_mask));
        if let Some(chart) = self.init_with_chart.clone() {
            app.insert_resource(GameChart::new(chart));
        }
    }
}

#[derive(Component)]
struct MaskBottom;

fn init_mask(mut commands: Commands) {
    commands
        .spawn((
            ShapeBundle {
                transform: Transform::from_xyz(900., 0., 10.),
                aabb: Aabb {
                    center: Vec3::new(0., 0., 0.).into(),
                    half_extents: Vec3::new(10000., 10000., 10000.).into(),
                },
                ..default()
            },
            Fill::default(),
        ))
        .insert(Name::new("mask_bottom"))
        .insert(MaskBottom);
}

const GRADIENT_NORMALIZED_HEIGHT: f32 = 0.05;
const RING_OFFSET: f32 = 0.2;
fn update_mask(
    mut mask: Query<(&mut Fill, &mut Path), With<MaskBottom>>,
    cams: Query<&OrthographicProjection, With<GameCamera>>,
    chart: Option<Res<GameChart>>,
    time: Res<GameTime>,
) {
    let game = cams.single();
    let area = game.area;
    let gradient_height = game.area.height() * GRADIENT_NORMALIZED_HEIGHT;
    let gradient_rect = Rect {
        min: Vec2 {
            x: area.min.x - area.width(),
            y: -gradient_height,
        },
        max: Vec2 {
            x: area.max.x + area.width(),
            y: 0.,
        },
    };
    let mask_non_transparent_rect = Rect {
        max: Vec2 {
            x: area.max.x + area.width(),
            y: -gradient_height,
        },
        min: Vec2 {
            x: area.min.x - area.width(),
            y: area.min.y - area.height(),
        },
    };
    let (mut fill, mut path) = mask.single_mut();
    fill.brush = {
        let gradient: Gradient = {
            let mut linear =
                LinearGradient::new_empty(Vec2::new(0., -gradient_height), Vec2::new(0., 0.));
            if let Some(color) = chart
                .map(|chart| {
                    chart
                        .theme_at(**time)
                        .ok()
                        .map(|t| colorrgba_to_color(t.this.color.background))
                })
                .flatten()
            {
                linear.add_stop(0., color.with_a(1.));
                linear.add_stop(1., color.with_a(0.));
            }
            linear.into()
        };
        gradient.into()
    };
    *path = GeometryBuilder::new()
        .add(&Rectangle::new(mask_non_transparent_rect))
        .add(&Rectangle::new(gradient_rect))
        .build();
}

mod rings;

fn spawn_game_camera(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                viewport_origin: [0.5, RING_OFFSET].into(),
                scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                    width: 900.,
                    height: 1600.,
                },
                ..default()
            },
            transform: Transform {
                translation: [900., 0., 999.0].into(),
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
    if !matches!(game_camera.target, RenderTarget::Image(_)) {
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
