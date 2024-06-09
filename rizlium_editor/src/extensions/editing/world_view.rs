use std::ops::ControlFlow;

use bevy::{
    math::vec2,
    prelude::*,
    render::render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
};
use bevy_egui::{EguiContexts, EguiUserTextures};
use egui::{Response, Sense, Ui};
use tools::Tool;

use crate::tab_system::{tab_focused, TabRegistrationExt};

use self::{
    cam_response::{
        ClickEventType, DragEventType, MouseEvent, MouseEventType, RaycastPlugin, ScreenMouseEvent,
    },
    tools::ToolsPlugin,
};

use super::tool_select_bar;

pub(super) mod cam_response;
pub(super) mod tools;
pub struct WorldViewPlugin;

impl Plugin for WorldViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            setup_world_cam.after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .add_plugins((RaycastPlugin, ToolsPlugin))
        .register_tab("edit.world_view", "World", world_tab, || true);
    }
}

pub fn edit_view_or_tool_focused() -> impl Condition<()>{
    tab_focused("edit.world_view").or_else(tab_focused("edit.tool_config"))
}

#[derive(Deref, Resource)]
pub struct WorldView(Handle<Image>);

#[derive(Component)]
pub struct WorldCam;

fn setup_world_cam(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    let handle = images.add(get_image());
    egui_context.add_image(handle.clone());
    commands.spawn((get_camera(handle.clone()), WorldCam));
    commands.insert_resource(WorldView(handle));
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

fn world_tab(
    In(ui): In<&mut Ui>,
    mut images: ResMut<Assets<Image>>,
    large_view: Res<WorldView>,
    textures: Res<EguiUserTextures>,
    mut camera: Query<(&mut OrthographicProjection, &mut Transform), With<WorldCam>>,
    mut scale: Local<Scale>,
    mut center: Local<Vec3>,
    mut event_writer: EventWriter<ScreenMouseEvent>,
    mut tool: ResMut<Tool>
) {
    let (mut projection, mut transform) = camera.single_mut();
    egui::TopBottomPanel::top("view_control").show_inside(ui, |ui| {
        ui.horizontal_centered(|ui| {
            **scale = 1. / projection.scale;
            if ui
                .add(egui::Slider::new(&mut **scale, 1e-2..=10.).logarithmic(true))
                .changed()
            {
                projection.scale = 1. / **scale;
            }
            *center = transform.translation;
            let mut changed = false;
            ui.label("x");
            changed |= ui.add(egui::DragValue::new(&mut center.x)).changed();
            ui.label("y");
            changed |= ui.add(egui::DragValue::new(&mut center.y)).changed();
            if changed {
                transform.translation = *center;
            }
        })
    });
    // resize img
    let Some(img) = images.get_mut(large_view.0.id()) else {
        return;
    };
    let size2d = ui.available_size_before_wrap();
    let rect = ui.available_rect_before_wrap();
    let pixel_size2d = size2d * 1.;
    let size = Extent3d {
        width: pixel_size2d.x as u32,
        height: pixel_size2d.y as u32,
        ..default()
    };
    img.resize(size);

    let img = textures.image_id(&large_view).expect("texture not found");
    // main img
    let area = ui.centered_and_justified(|ui| ui.add(egui::Image::new((img, size2d)))).response.rect;
    // tool select
    tool_select_bar::tool_select_bar(ui, area.left_top() + [10.,10.].into(), &mut tool);
    let response = ui.interact(rect, ui.next_auto_id(), Sense::click_and_drag());
    ui.ctx().input(|input| {
        if response.contains_pointer() || response.interact_pointer_pos().is_some() {
            if let Some(pos) = input.pointer.hover_pos() {
                let releative_pos = pos - rect.left_top();
                let releative_pos = egui_to_glam(releative_pos);
                event_writer.send(ScreenMouseEvent(MouseEvent {
                    event_type: get_event_type(
                        &response,
                        if response.dragged() {
                            input.pointer.delta()
                        } else {
                            egui::Vec2::ZERO
                        },
                    ),
                    button: response
                        .interact_pointer_pos()
                        .is_some()
                        .then(|| {
                            iter_pointer(response.triple_clicked)
                                .or_else(|| iter_pointer(response.double_clicked))
                                .or_else(|| iter_pointer(response.clicked))
                        })
                        .flatten(),
                    pos: releative_pos.extend(0.),
                }));
            }
        }
    })
}

fn egui_to_glam(vec2: egui::Vec2) -> Vec2 {
    Vec2::new(vec2.x, vec2.y)
}

fn iter_pointer(pointer: [bool; egui::NUM_POINTER_BUTTONS]) -> Option<MouseButton> {
    if let ControlFlow::Break(button) = pointer
        .iter()
        .zip([
            MouseButton::Left,
            MouseButton::Right,
            MouseButton::Middle,
            MouseButton::Other(0),
            MouseButton::Other(1),
        ])
        .try_for_each(|(is_clicked, button)| {
            if !*is_clicked {
                ControlFlow::Continue(())
            } else {
                ControlFlow::Break(button)
            }
        })
    {
        Some(button)
    } else {
        None
    }
}

fn get_event_type(response: &Response, drag_delta: egui::Vec2) -> MouseEventType {
    if any_true(&response.triple_clicked) {
        MouseEventType::Click(ClickEventType::Triple)
    } else if any_true(&response.double_clicked) {
        MouseEventType::Click(ClickEventType::Double)
    } else if any_true(&response.clicked) {
        MouseEventType::Click(ClickEventType::Single)
    } else if response.drag_started() {
        MouseEventType::Drag(DragEventType::DragStarted)
    } else if response.dragged() {
        MouseEventType::Drag(DragEventType::Dragging(
            egui_to_glam(drag_delta) * vec2(1., -1.),
        )) // flip y axis
    } else if response.drag_released() {
        MouseEventType::Drag(DragEventType::DragEnded)
    } else {
        MouseEventType::Hover
    }
}

fn any_true(slice: &[bool]) -> bool {
    slice.iter().any(|i| *i)
}
