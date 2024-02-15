use bevy::{
    prelude::*,
    render::{
        camera::ScalingMode,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use bevy_egui::{EguiContexts, EguiUserTextures};
use egui::{Color32, Ui};

use crate::tab_system::TabRegistrationExt;
pub struct LargeGameCamPlugin;

impl Plugin for LargeGameCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            setup_large_game_cam.after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .register_tab("edit.large_cam".into(), "World", large_game_cam_tab, || {
            true
        });
    }
}

#[derive(Deref, Resource)]
pub struct LargeGameView(Handle<Image>);

#[derive(Component)]
pub struct LargeGameCam;

fn setup_large_game_cam(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(get_image());
    egui_context.add_image(handle.clone());
    commands.spawn((get_camera(handle.clone()), LargeGameCam));
    commands.insert_resource(LargeGameView(handle));
}

fn get_image() -> Image {
    // This is the texture that will be rendered to.
    Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: default(),
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    }
}

fn get_camera(handle: Handle<Image>) -> Camera2dBundle {
    // todo: use a shader to shadow places which are not in GameView
    Camera2dBundle {
        camera: Camera {
            target: bevy::render::camera::RenderTarget::Image(handle),
            ..default()
        },
        ..default()
    }
}

#[derive(Deref, DerefMut)]
struct Scale(f32);

impl Default for Scale {
    fn default() -> Self {
        Self(1.)
    }
}

fn large_game_cam_tab(
    In(ui): In<&mut Ui>,
    mut images: ResMut<Assets<Image>>,
    large_view: Res<LargeGameView>,
    textures: Res<EguiUserTextures>,
    mut camera: Query<(&mut OrthographicProjection, &mut Transform), With<LargeGameCam>>,
    mut scale: Local<Scale>,
    mut center: Local<Vec3>,
) {
    egui::TopBottomPanel::top("view_control").show_inside(ui, |ui| {
        ui.horizontal_centered(|ui| {
            ui.add(egui::Slider::new(&mut **scale, 1e-2..=10.).logarithmic(true));
            ui.label("x");
            ui.add(egui::DragValue::new(&mut center.x));
            ui.label("y");
            ui.add(egui::DragValue::new(&mut center.y));
        })
    });
    // resize img
    let Some(img) = images.get_mut(large_view.0.id()) else {
        return;
    };
    let size2d = ui.available_size_before_wrap();
    let pixel_size2d = Vec2::new(
        size2d.x * ui.ctx().pixels_per_point(),
        size2d.y * ui.ctx().pixels_per_point(),
    );
    let size = Extent3d {
        width: pixel_size2d.x as u32,
        height: pixel_size2d.y as u32,
        ..default()
    };
    img.resize(size);

    let (mut projection, mut transform) = camera.single_mut();
    projection.scaling_mode = ScalingMode::WindowSize(scale.0);
    transform.translation = *center;

    let img = textures.image_id(&large_view).expect("texture not found");

    ui.centered_and_justified(|ui| ui.add(egui::Image::new((img, size2d))));
}
