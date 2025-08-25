use bevy::{
    input::mouse::MouseMotion,
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_egui::{EguiContexts, EguiUserTextures};
use egui::{DragValue, Ui, Widget};
use helium_framework::prelude::{tab_focused, ActionsExt, Hotkey, HotkeysExt, TabRegistrationExt, TriggerType};

/// Mouse sensitivity and movement speed
#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

/// Key configuration
#[derive(Resource)]
pub struct KeyBindings {
    pub move_forward: KeyCode,
    pub move_backward: KeyCode,
    pub move_left: KeyCode,
    pub move_right: KeyCode,
    pub move_ascend: KeyCode,
    pub move_descend: KeyCode,
    pub toggle_grab_cursor: KeyCode,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            move_forward: KeyCode::KeyW,
            move_backward: KeyCode::KeyS,
            move_left: KeyCode::KeyA,
            move_right: KeyCode::KeyD,
            move_ascend: KeyCode::Space,
            move_descend: KeyCode::ShiftLeft,
            toggle_grab_cursor: KeyCode::Escape,
        }
    }
}

/// Used in queries when you want flycams and not other cameras
/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    info!("Toggling cursor grab mode: {:?}", window.cursor_options.grab_mode);
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = false;
        }
        _ => {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

#[derive(Resource)]
pub struct DebugCam(Handle<Image>);

/// Spawns the `Camera3dBundle` to be controlled
fn setup_player(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(get_image());
    egui_context.add_image(handle.clone());
    commands.spawn(get_cam(handle.clone()));
    commands.insert_resource(DebugCam(handle));
}

fn get_cam(image: Handle<Image>) -> impl Bundle {
    (
        Camera2d,
        FlyCam,
        Camera {
            target: RenderTarget::Image(image.into()),
            ..Default::default()
        },
        Projection::Perspective(PerspectiveProjection::default()),
        Transform::from_xyz(0.0, 0.0, -500.0).looking_at(Vec3::ZERO, Vec3::Y),
    )
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

/// Handles moving forward
fn player_move_forward(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                let local_z = transform.local_z();
                let forward = -Vec3::new(local_z.x, 0., local_z.z);
                transform.translation +=
                    forward.normalize_or_zero() * time.delta_secs() * settings.speed;
            }
        }
    }
}

// Handles moving backward
fn player_move_backward(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                let local_z = transform.local_z();
                let forward = -Vec3::new(local_z.x, 0., local_z.z);
                transform.translation -=
                    forward.normalize_or_zero() * time.delta_secs() * settings.speed;
            }
        }
    }
}

// Handles moving left
fn player_move_left(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                let local_z = transform.local_z();
                let right = Vec3::new(local_z.z, 0., -local_z.x);
                transform.translation -=
                    right.normalize_or_zero() * time.delta_secs() * settings.speed;
            }
        }
    }
}

// Handles moving right
fn player_move_right(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                let local_z = transform.local_z();
                let right = Vec3::new(local_z.z, 0., -local_z.x);
                transform.translation +=
                    right.normalize_or_zero() * time.delta_secs() * settings.speed;
            }
        }
    }
}

// Handles ascending
fn player_move_ascend(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                transform.translation += Vec3::Y * time.delta_secs() * settings.speed;
            }
        }
    }
}

// Handles descending
fn player_move_descend(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                transform.translation -= Vec3::Y * time.delta_secs() * settings.speed;
            }
        }
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Ok(window) = primary_window.single() {
        for mut transform in query.iter_mut() {
            for ev in state.read() {
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                pitch = pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// Same as [`PlayerPlugin`] but does not spawn a camera
pub struct DebugCamExtension;
impl Plugin for DebugCamExtension {
    fn build(&self, app: &mut App) {
        app.reflect_system("player.move_forward", "", player_move_forward)
            .reflect_system("player.move_backward", "", player_move_backward)
            .reflect_system("player.move_left", "", player_move_left)
            .reflect_system("player.move_right", "", player_move_right)
            .reflect_system("player.move_ascend", "", player_move_ascend)
            .reflect_system("player.move_descend", "", player_move_descend)
            .reflect_system("player.look", "", player_look)
            .reflect_system("cursor.grab", "", cursor_grab);
        app.register_hotkey(
            "player.move_forward",
            [Hotkey::new_advanced(
                [KeyCode::KeyW],
                tab_focused("debug_flycam"),
                TriggerType::Repeat,
            )],
        )
        .register_hotkey(
            "player.move_backward",
            [Hotkey::new_advanced(
                [KeyCode::KeyS],
                tab_focused("debug_flycam"),
                TriggerType::Repeat,
            )],
        )
        .register_hotkey(
            "player.move_left",
            [Hotkey::new_advanced(
                [KeyCode::KeyA],
                tab_focused("debug_flycam"),
                TriggerType::Repeat,
            )],
        )
        .register_hotkey(
            "player.move_right",
            [Hotkey::new_advanced(
                [KeyCode::KeyD],
                tab_focused("debug_flycam"),
                TriggerType::Repeat,
            )],
        )
        .register_hotkey(
            "player.move_ascend",
            [Hotkey::new_advanced(
                [KeyCode::Space],
                tab_focused("debug_flycam"),
                TriggerType::Repeat,
            )],
        )
        .register_hotkey(
            "player.move_descend",
            [Hotkey::new_advanced(
                [KeyCode::ShiftLeft],
                tab_focused("debug_flycam"),
                TriggerType::Repeat,
            )],
        )
        .register_hotkey(
            "cursor.grab",
            [Hotkey::new_advanced(
                [KeyCode::KeyG],
                tab_focused("debug_flycam"),
                TriggerType::Pressed,
            )],
        );

        app.init_resource::<MovementSettings>()
            // .init_resource::<KeyBindings>()
            .add_systems(
                PreStartup,
                setup_player.after(bevy_egui::EguiStartupSet::InitContexts),
            )
            .add_systems(Update, player_look);

        app.register_tab(
            "debug_flycam",
            "Debug Flycam",
            debug_cam_tab,|| true);
    }
}

fn debug_cam_tab(
    InMut(ui): InMut<'_, Ui>,
    image: Res<DebugCam>,
    mut images: ResMut<Assets<Image>>,

    textures: Res<EguiUserTextures>,
    mut setting: ResMut<MovementSettings>,
) {
    // resize img
    let Some(img) = images.get_mut(image.0.id()) else {
        return;
    };
    DragValue::new(&mut setting.sensitivity)
        .speed(0.00001)
        .ui(ui);
    DragValue::new(&mut setting.speed)
        .speed(0.1)
        .ui(ui);

    let size2d = ui.available_size_before_wrap();
    let rect = ui.available_rect_before_wrap();
    let pixel_size2d = size2d * 1.;
    let size = Extent3d {
        width: pixel_size2d.x as u32,
        height: pixel_size2d.y as u32,
        ..default()
    };
    img.resize(size);

    let img = textures.image_id(&image.0).expect("texture not found");
    ui.centered_and_justified(|ui| ui.add(egui::Image::new((img, size2d))));
}
