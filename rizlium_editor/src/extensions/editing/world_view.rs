use std::ops::ControlFlow;

use bevy::{
    math::vec2,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
        view::RenderLayers,
    },
};
use bevy_egui::{EguiContexts, EguiUserTextures};
use bevy_prototype_lyon::{
    draw::Stroke,
    entity::ShapeBundle,
    prelude::GeometryBuilder,
    shapes::Circle as Circle0,
};
use egui::{InputState, PointerButton, Response, Sense, Ui};
use rizlium_render::{GameChart, GameChartCache, GameTime};
use rust_i18n::t;
use tools::Tool;

use helium_framework::prelude::*;

use self::{
    cam_response::{
        ClickEventType, DragEventType, MouseEvent, MouseEventType, RaycastPlugin, ScreenMouseEvent,
    },
    tools::ToolsPlugin,
};

use super::tool_select_bar;

pub mod cam_response;
pub(super) mod tools;
pub struct WorldViewPlugin;

impl Plugin for WorldViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            setup_world_cam.after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .add_plugins((RaycastPlugin, ToolsPlugin, PointIndicatorPlugin))
        .register_tab(
            "edit.world_view",
            t!("edit.world_view.tab"),
            world_tab,
            || true,
        );
    }
}

pub fn edit_view_or_tool_focused() -> impl Condition<()> {
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

fn get_camera(handle: Handle<Image>) -> impl Bundle {
    let layers = RenderLayers::default().with(114);
    (
        // todo: use a shader to shadow places which are not in GameView
        Camera2d,
        Camera {
            target: bevy::render::camera::RenderTarget::Image(handle),
            ..default()
        },
        layers,
    )
}

#[derive(Deref, DerefMut)]
struct Scale(f32);

impl Default for Scale {
    fn default() -> Self {
        Self(1.)
    }
}

fn world_tab(
    InMut(ui): InMut<Ui>,
    mut images: ResMut<Assets<Image>>,
    large_view: Res<WorldView>,
    textures: Res<EguiUserTextures>,
    mut camera: Query<(&mut OrthographicProjection, &mut Transform), With<WorldCam>>,
    mut scale: Local<Scale>,
    mut center: Local<Vec3>,
    mut event_writer: EventWriter<ScreenMouseEvent>,
    mut tool: ResMut<Tool>,
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
    let area = ui
        .centered_and_justified(|ui| ui.add(egui::Image::new((img, size2d))))
        .response
        .rect;
    // tool select
    tool_select_bar::tool_select_bar(ui, area.left_top() + [10., 10.].into(), &mut tool);
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
                        input,
                    ),
                    button: response
                        .interact_pointer_pos()
                        .is_some()
                        .then(|| iter_pointer(|b| input.pointer.button_clicked(b)))
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

fn iter_pointer(mut check: impl FnMut(PointerButton) -> bool) -> Option<MouseButton> {
    if let ControlFlow::Break(button) = [
        MouseButton::Left,
        MouseButton::Right,
        MouseButton::Middle,
        MouseButton::Other(0),
        MouseButton::Other(1),
    ]
    .into_iter()
    .zip([
        PointerButton::Primary,
        PointerButton::Secondary,
        PointerButton::Middle,
        PointerButton::Extra1,
        PointerButton::Extra2,
    ])
    .try_for_each(|(bevy_button, egui_button)| {
        if !check(egui_button) {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(bevy_button)
        }
    }) {
        Some(button)
    } else {
        None
    }
}

fn get_event_type(
    response: &Response,
    drag_delta: egui::Vec2,
    input: &InputState,
) -> MouseEventType {
    if iter_pointer(|b| input.pointer.button_triple_clicked(b)).is_some() {
        MouseEventType::Click(ClickEventType::Triple)
    } else if iter_pointer(|b| input.pointer.button_double_clicked(b)).is_some() {
        MouseEventType::Click(ClickEventType::Double)
    } else if response.clicked {
        MouseEventType::Click(ClickEventType::Single)
    } else if response.drag_started() {
        MouseEventType::Drag(DragEventType::DragStarted)
    } else if response.dragged() {
        MouseEventType::Drag(DragEventType::Dragging(
            egui_to_glam(drag_delta) * vec2(1., -1.),
        )) // flip y axis
    } else if response.drag_stopped() {
        MouseEventType::Drag(DragEventType::DragEnded)
    } else {
        MouseEventType::Hover
    }
}

// 长类型让我抓狂
macro_rules! chart_update {
    () => {
        resource_exists::<GameChart>.and_then(
            resource_exists_and_changed::<GameChart>.or_else(resource_changed::<GameTime>),
        )
    };
}

struct PointIndicatorPlugin;

impl Plugin for PointIndicatorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreUpdate,
            (add_points_indicator, associate_segment)
                .run_if(resource_exists_and_changed::<GameChart>),
        )
        .add_systems(Update, (update_shape).run_if(chart_update!()));
    }
}

#[derive(Component, Default)]
struct PointIndicator;

#[derive(Component, Default)]
struct PointIndicatorId {
    line_idx: usize,
    keypoint_idx: usize,
}

#[derive(Bundle, Default)]
pub struct PointIndicatorBundle {
    layer: RenderLayers,
    line: PointIndicator,
    shape: ShapeBundle,
    stroke: Stroke,
    cam_response: cam_response::CamResponse,
}

fn add_points_indicator(
    mut commands: Commands,
    chart: Res<GameChart>,
    indicators: Query<&PointIndicator>,
) {
    let segment_count = chart.segment_count();
    let now_count = indicators.iter().count();
    let delta = segment_count - now_count;
    debug!("attempting to add {delta} indicators");
    for _ in now_count..segment_count {
        commands.spawn(PointIndicatorBundle {
            shape: ShapeBundle {
                path: GeometryBuilder::new()
                    .add(&Circle0 {
                        radius: 10.,
                        center: [0., 0.].into(),
                    })
                    .build(),
                transform: Transform::from_translation(Vec3 {
                    x: 0.,
                    y: 0.,
                    z: 20.,
                }),
                ..default()
            },
            stroke: Stroke::new(Color::BLACK, 10.),
            layer: RenderLayers::from_layers(&[114]),
            ..Default::default()
        });
    }
}

fn associate_segment(
    mut commands: Commands,
    chart: Res<GameChart>,
    lines: Query<Entity, With<PointIndicator>>,
) {
    debug!("running system assocate_segment");
    // return_nothing_change!(chart);
    for (entity, (line_idx, keypoint_idx)) in lines.iter().zip(chart.iter_segment()) {
        commands.entity(entity).insert(PointIndicatorId {
            line_idx,
            keypoint_idx,
        });
    }
}

fn update_shape(
    chart: Res<GameChart>,
    cache: Res<GameChartCache>,
    time: Res<GameTime>,
    mut lines: Query<(&mut Stroke, &PointIndicatorId, &mut Transform)>,
) {
    lines
        .par_iter_mut()
        // .batching_strategy(BatchingStrategy::new().batches_per_thread(100))
        .for_each(|(_, id, mut transform)| {
            let line_idx = id.line_idx;
            let keypoint_idx = id.keypoint_idx;
            let Some(pos1) =
                chart
                    .with_cache(&cache)
                    .pos_for_linepoint_at(line_idx, keypoint_idx, **time)
            else {
                return;
            };
            let pos1: Vec2 = pos1.into();
            transform.translation = pos1.extend(transform.translation.z);
        });
}
