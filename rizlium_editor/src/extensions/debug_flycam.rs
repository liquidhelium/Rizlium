// 引入 Bevy 引擎的相关模块。
use bevy::{
    input::mouse::MouseMotion, // 鼠标移动事件。
    prelude::*, // Bevy 核心预设。
    render::{
        camera::RenderTarget, // 相机渲染目标。
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, // 渲染纹理相关的定义。
        },
    },
    window::{CursorGrabMode, PrimaryWindow}, // 窗口和光标控制。
};
// 引入 `bevy_egui` 插件，用于在 egui 中显示 Bevy 纹理。
use bevy_egui::{EguiContexts, EguiUserTextures};
// 引入 egui 库的组件。
use egui::{DragValue, Ui, Widget};
// 引入自定义的 `helium_framework` 框架。
use helium_framework::prelude::{tab_focused, ActionsExt, Hotkey, HotkeysExt, TabRegistrationExt, TriggerType};

/// `MovementSettings` 是一个 Bevy 资源，用于存储飞行相机的移动和旋转速度设置。
#[derive(Resource)]
pub struct MovementSettings {
    pub sensitivity: f32, // 鼠标灵敏度。
    pub speed: f32, // 相机移动速度。
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 12.,
        }
    }
}

/// `KeyBindings` 是一个（当前未使用）的资源，用于定义相机控制的按键绑定。
/// 目前，按键绑定是通过 `helium_framework` 的热键系统硬编码的。
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

/// `FlyCam` 是一个标记组件（marker component）。
/// 它的作用是区分我们创建的飞行相机和其他可能存在的相机实体。
/// 在查询（Query）时，我们可以通过 `With<FlyCam>` 来确保只选择飞行相机。
#[derive(Component)]
pub struct FlyCam;

/// `toggle_grab_cursor` 函数用于切换鼠标光标的抓取模式。
/// 当光标被抓取时，它会被限制在窗口内并且不可见，这对于第一人称视角的控制是必需的。
fn toggle_grab_cursor(window: &mut Window) {
    info!("Toggling cursor grab mode: {:?}", window.cursor_options.grab_mode);
    match window.cursor_options.grab_mode {
        CursorGrabMode::None => {
            window.cursor_options.grab_mode = CursorGrabMode::Confined;
            window.cursor_options.visible = false;
        }
        // 否则，切换回非抓取模式。
        _ => {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

/// `initial_grab_cursor` 是一个（当前未使用）的启动系统，用于在游戏开始时自动抓取光标。
fn initial_grab_cursor(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// `DebugCam` 是一个 Bevy 资源，存储了飞行相机渲染目标的图像句柄（`Handle<Image>`）。
#[derive(Resource)]
pub struct DebugCam(Handle<Image>);

/// `setup_player` 是一个 Bevy 启动系统，负责创建飞行相机实体和其渲染目标。
fn setup_player(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    // 创建一个新的 Bevy 图像资源作为渲染目标。
    let handle = images.add(get_image());
    // 将这个图像句柄注册到 egui 中，以便 egui 可以渲染它。
    egui_context.add_image(handle.clone());
    // 生成相机实体，并将其渲染目标设置为我们刚刚创建的图像。
    commands.spawn(get_cam(handle.clone()));
    // 将图像句柄存储在 `DebugCam` 资源中，以便 UI 系统可以访问它。
    commands.insert_resource(DebugCam(handle));
}

/// `get_cam` 是一个辅助函数，用于构建相机实体所需的组件 `Bundle`。
fn get_cam(image: Handle<Image>) -> impl Bundle {
    (
        Camera2d,
        FlyCam,
        Camera {
            target: RenderTarget::Image(image.into()),
            ..Default::default()
        },
        // 虽然是 2D 相机，但我们使用透视投影来获得 3D 效果。
        Projection::Perspective(PerspectiveProjection::default()),
        // 设置相机的初始位置和朝向。
        Transform::from_xyz(0.0, 0.0, -500.0).looking_at(Vec3::ZERO, Vec3::Y),
    )
}

/// `get_image` 是一个辅助函数，用于创建一个空的 Bevy `Image` 资源。
fn get_image() -> Image {
    // 这个图像将作为渲染目标纹理。
    Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size: default(), // 初始大小为空，将在 UI 系统中根据窗口大小动态调整。
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb, // 标准的颜色格式。
            mip_level_count: 1,
            sample_count: 1,
            // 设置纹理用途：可以被着色器绑定、可以作为拷贝目标、可以作为渲染附件。
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    }
}

/// `player_move_forward` 是一个 Bevy 系统，用于处理相机的前进移动。
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
                // 我们只在水平面上移动，所以忽略 Y 分量。
                let forward = -Vec3::new(local_z.x, 0., local_z.z);
                // 根据时间增量和速度来更新相机的位置。
                transform.translation +=
                    forward.normalize_or_zero() * time.delta_secs() * settings.speed;
            }
        }
    }
}

// `player_move_backward` 系统，处理后退移动，逻辑与前进类似，只是方向相反。
fn player_move_backward(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.get_single() {
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

// `player_move_left` 系统，处理向左平移。
fn player_move_left(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                // 通过 `local_z` 的分量交换和变号来计算出右方向向量。
                let local_z = transform.local_z();
                let right = Vec3::new(local_z.z, 0., -local_z.x);
                // 向右的相反方向移动即为向左。
                transform.translation -=
                    right.normalize_or_zero() * time.delta_secs() * settings.speed;
            }
        }
    }
}

// `player_move_right` 系统，处理向右平移。
fn player_move_right(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.get_single() {
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

// `player_move_ascend` 系统，处理垂直向上移动。
fn player_move_ascend(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                transform.translation += Vec3::Y * time.delta_secs() * settings.speed;
            }
        }
    }
}

// `player_move_descend` 系统，处理垂直向下移动。
fn player_move_descend(
    time: Res<Time>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    settings: Res<MovementSettings>,
    mut query: Query<(&FlyCam, &mut Transform)>,
) {
    if let Ok(window) = primary_window.get_single() {
        for (_camera, mut transform) in query.iter_mut() {
            if window.cursor_options.grab_mode != CursorGrabMode::None {
                transform.translation -= Vec3::Y * time.delta_secs() * settings.speed;
            }
        }
    }
}

/// `player_look` 系统处理鼠标移动事件，用于旋转相机视角。
fn player_look(
    settings: Res<MovementSettings>,
    primary_window: Query<&Window, With<PrimaryWindow>>,
    mut state: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Ok(window) = primary_window.get_single() {
        for mut transform in query.iter_mut() {
            // 遍历所有鼠标移动事件。
            for ev in state.read() {
                // 将当前的旋转（四元数）转换为欧拉角（YXZ顺序），方便修改 yaw 和 pitch。
                let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);
                match window.cursor_options.grab_mode {
                    CursorGrabMode::None => (),
                    _ => {
                        // 只有在光标被抓取时才旋转。
                        // `window_scale` 用于使灵敏度在不同窗口大小下保持一致。
                        let window_scale = window.height().min(window.width());
                        pitch -= (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                        yaw -= (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                    }
                }

                // 限制 pitch（俯仰角）的范围，防止相机上下翻转。
                pitch = pitch.clamp(-1.54, 1.54);

                // 从修改后的 yaw 和 pitch 重新构建旋转四元数。
                // 这个顺序很重要，可以防止不希望的“翻滚”（roll）。
                transform.rotation =
                    Quat::from_axis_angle(Vec3::Y, yaw) * Quat::from_axis_angle(Vec3::X, pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

/// `cursor_grab` 系统用于响应快捷键，切换光标抓取状态。
fn cursor_grab(mut primary_window: Query<&mut Window, With<PrimaryWindow>>) {
    if let Ok(mut window) = primary_window.get_single_mut() {
        toggle_grab_cursor(&mut window);
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// `DebugCamExtension` 是主要的 Bevy 插件，它将所有与飞行相机相关的系统和资源组合在一起。
pub struct DebugCamExtension;
impl Plugin for DebugCamExtension {
    fn build(&self, app: &mut App) {
        // 注册所有移动和观察的系统为反射系统，以便可以通过热键系统调用它们。
        app.reflect_system("player.move_forward", "", player_move_forward)
            .reflect_system("player.move_backward", "", player_move_backward)
            .reflect_system("player.move_left", "", player_move_left)
            .reflect_system("player.move_right", "", player_move_right)
            .reflect_system("player.move_ascend", "", player_move_ascend)
            .reflect_system("player.move_descend", "", player_move_descend)
            .reflect_system("player.look", "", player_look)
            .reflect_system("cursor.grab", "", cursor_grab);
        // 为每个移动动作注册热键。
        // `Hotkey::new_advanced` 允许我们指定一个条件（`tab_focused`）和一个触发类型（`Repeat`）。
        // 这意味着只有当 "debug_flycam" 标签页获得焦点时，按住这些键才会持续触发移动。
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
                TriggerType::Pressed, // 按下时触发一次。
            )],
        );

        app.init_resource::<MovementSettings>()
            .add_systems(
                PreStartup,
                // 在 egui 上下文初始化之后运行 `setup_player`。
                setup_player.after(bevy_egui::EguiStartupSet::InitContexts),
            )
            .add_systems(Update, player_look); // `player_look` 需要每帧运行以响应鼠标事件。

        // 注册 "debug_flycam" 标签页。
        app.register_tab(
            "debug_flycam",
            "Debug Flycam",
            debug_cam_tab,|| true);
    }
}

/// `debug_cam_tab` 是一个 Bevy `widget` 系统，负责渲染 "Debug Flycam" 标签页的内容。
fn debug_cam_tab(
    InMut(ui): InMut<'_, Ui>,
    image: Res<DebugCam>, // 获取渲染目标的图像句柄。
    mut images: ResMut<Assets<Image>>, // 获取所有图像资源的访问权限。

    textures: Res<EguiUserTextures>, // 用于将 Bevy 图像句柄转换为 egui 纹理 ID。
    mut setting: ResMut<MovementSettings>, // 获取移动设置以供 UI 修改。
) {
    // --- 渲染设置控件 ---
    let Some(img) = images.get_mut(image.0.id()) else {
        return;
    };
    // 添加一个 `DragValue` 小部件来调整灵敏度。
    DragValue::new(&mut setting.sensitivity)
        .speed(0.00001)
        .ui(ui);
    // 添加一个 `DragValue` 小部件来调整速度。
    DragValue::new(&mut setting.speed)
        .speed(0.1)
        .ui(ui);

    // --- 调整渲染目标大小并渲染 ---
    // 获取标签页内可用的 UI 空间大小。
    let size2d = ui.available_size_before_wrap();
    // 将 UI 空间大小转换为像素大小。
    let pixel_size2d = size2d * 1.; // 乘以 UI 的 `pixels_per_point`。
    let size = Extent3d {
        width: pixel_size2d.x as u32,
        height: pixel_size2d.y as u32,
        ..default()
    };
    // 调整渲染目标图像的大小以匹配 UI 空间的大小。
    img.resize(size);

    // 从 `EguiUserTextures` 中获取 egui 可以使用的纹理 ID。
    let img_id = textures.image_id(&image.0).expect("texture not found");
    // 在 UI 中居中并最大化地显示渲染结果图像。
    ui.centered_and_justified(|ui| ui.add(egui::Image::new((img_id, size2d))));
}
