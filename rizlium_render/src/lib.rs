use bevy::{prelude::*, render::camera::RenderTarget};

use bevy_prototype_lyon::{prelude::*, shapes::Circle};
use chart_loader::ChartLoadingPlugin;
use rizlium_chart::{chart::Chart, prelude::{ColorRGBA}, VIEW_RECT};

use theme::BackgroundThemePlugin;
use time_and_audio::TimeAndAudioPlugin;
pub use time_and_audio::TimeManager;

mod chart;
mod line_rendering;
pub use line_rendering::ShowLines;
mod theme;
mod time_and_audio;
mod chart_loader;
pub use chart_loader::LoadChartEvent;

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
            ))
            .add_systems(Startup, spawn_game_camera)
            .add_systems(PostUpdate, (bind_gameview,( rings, add_rings).run_if(resource_exists::<GameChart>())));
        if let Some(chart) = self.init_with_chart.clone() {
            app.insert_resource(GameChart::new(chart));
        }
    }
}

#[derive(Component)]
struct Ring(usize);

fn rings(chart: Res<GameChart>,cache: Res<GameChartCache>, time: Res<GameTime>, mut rings: Query<(&mut Transform, &mut Visibility, &Ring)>) {
    rings.par_iter_mut().for_each_mut(|(mut transform,mut vis,ring)| {
        // info!("processing ring {}",ring.0);
        let chart_with_cache = chart.with_cache(&cache);
        let Some(pos) = chart_with_cache.line_pos_at(ring.0, **time, **time) else {
            *vis = Visibility::Hidden;
            return;
        };
        *vis = Visibility::Visible;
        *transform = transform.with_translation(Vec2::from(pos).extend(0.));
    })
}

fn add_rings(mut commands: Commands, chart: Res<GameChart>, rings: Query<&Ring>) {
    for i in rings.iter().count()..chart.lines.len() {
        // info!("adding ring {}", i);
        commands.spawn(
            (ShapeBundle {
                path: GeometryBuilder::new().add(&Circle {
                    radius: 50.,
                    center: [0.,0.].into()
                }).build(),
                transform: Transform::from_translation(Vec3 { x: 0., y: 0., z: 10. }),
                ..default()
            },
            Stroke::new(Color::BLACK, 10.),
            Ring(i))
        );
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
