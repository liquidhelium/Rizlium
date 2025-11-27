use rust_i18n::t;
use strum::EnumIter;

use bevy::{input::mouse::MouseWheel, prelude::*};
use egui::Ui;
use rizlium_chart::{
    chart::{ColorRGBA, KeyPoint, Line, LinePointData},
    editing::{
        chart_path::{LinePath, LinePointPath},
        commands::{CommandSequence, EditPoint, InsertLine, InsertPoint, Nop},
    },
};
use rizlium_render::{ChartLineId, ChartProvider as _};

use self::tool_configs::{PencilToolConfig, ToolConfigExt};
use crate::{
    extensions::{
        editing::{world_view::WorldViewConfig, ChartEditHistory},
        inspector::{ChartItem, SelectedItem},
    },
    project::ProjectState,
    utils::WorldToGame,
};
use helium_framework::prelude::*;

use super::{
    cam_response::{DragEventType, MouseEvent, MouseEventType, ScreenMouseEvent, WorldMouseEvent},
    edit_view_or_tool_focused,
    snapping::{SnappingConfig, SnappingContext},
    PointIndicatorId, WorldCam,
};

pub fn is_tool(tool: Tool) -> impl Condition<()> {
    edit_view_or_tool_focused().and(resource_exists_and_equals(tool))
}

pub fn previous_tool(tool: Tool) -> impl Condition<()> {
    resource_exists_and_equals(OriginalTool(Some(tool))).and(|| true)
}

/// Control the switching of tools and some individual tools.
pub struct ToolsPlugin;

impl Plugin for ToolsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Tool>()
            .init_resource::<OriginalTool>()
            .add_event::<DiscardPreeditEvent>()
            .init_tool_config::<tool_configs::PencilToolConfig>()
            .add_systems(
                Update,
                (view_tool, pencil_tool, select_tool)
                    .run_if(crate::ui::tab_opened("edit.world_view")),
            );
        app.reflect_system(
            "edit.world_view.temp_toggle_view",
            t!("edit.world_view.temp_toggle_view"),
            temp_toggle_view,
        );
        app.reflect_system(
            "edit.world_view.to_pencil",
            t!("edit.world_view.to_pencil.desc"),
            switch_tool(Tool::Pencil),
        );
        app.reflect_system(
            "edit.world_view.to_select",
            t!("edit.world_view.to_select.desc"),
            switch_tool(Tool::Select),
        );
        app.reflect_system(
            "edit.discard_preedit",
            t!("edit.discard_preedit"),
            discard_preedit,
        );
        app.register_hotkey(
            "edit.world_view.to_pencil",
            [Hotkey::new([KeyCode::KeyP], edit_view_or_tool_focused())],
        )
        .register_hotkey(
            "edit.world_view.temp_toggle_view",
            [Hotkey::new_advanced(
                [KeyCode::AltLeft],
                is_tool(Tool::Pencil).or(previous_tool(Tool::Pencil).and(is_tool(Tool::View))),
                TriggerType::PressAndRelease,
            )],
        );
        app.register_hotkey(
            "edit.world_view.to_select",
            [Hotkey::new([KeyCode::KeyS], edit_view_or_tool_focused())],
        );
        app.register_hotkey(
            "edit.discard_preedit",
            [Hotkey::new_global([KeyCode::Escape])],
        );
    }
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug, EnumIter)]
pub enum Tool {
    #[default]
    View,
    Pencil,
    Select,
}

#[derive(Event, Default)]
pub struct DiscardPreeditEvent;

fn discard_preedit(mut ev: EventWriter<DiscardPreeditEvent>) {
    ev.write_default();
}

impl Tool {
    pub fn config_ui(&self, ui: &mut Ui, world: &mut World) {
        if self == &Self::Pencil {
            tool_configs::show_window::<tool_configs::PencilToolConfig>(ui, world)
        }
    }
}

mod tool_configs;

impl Tool {
    pub fn set(&mut self, tool: Tool) {
        debug!("Switching to tool {tool:?}");
        *self = tool;
    }
}

#[derive(Resource, Default, PartialEq, Eq, Deref, DerefMut, Debug)]
pub struct OriginalTool(Option<Tool>);

const SCROLL_SPEED: f32 = 1e-2;

const fn switch_tool(tool: Tool) -> impl FnMut(ResMut<Tool>) {
    move |mut res: ResMut<Tool>| res.set(tool)
}

fn view_tool(
    mut events: EventReader<ScreenMouseEvent>,
    mut camera: Query<(&mut Projection, &mut Transform), With<WorldCam>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    tool: Res<Tool>,
) -> Result<()> {
    if *tool != Tool::View {
        mouse_wheel.clear();
        events.clear();
        return Ok(());
    }
    let (mut projection, mut transform) = camera.single_mut()?;
    let Projection::Orthographic(projection) = &mut *projection else {
        return Ok(());
    };
    let mut scale: f32 = 1. / projection.scale;
    if !events.is_empty() {
        mouse_wheel.read().for_each(|event| {
            //取对进行更丝滑的过渡
            scale = scale.ln();
            scale += event.y * SCROLL_SPEED;
            scale = scale.exp();
            scale = scale.clamp(1e-2, 10.);
        });
        projection.scale = 1. / scale;
    }
    events.read().for_each(|event| {
        if let ScreenMouseEvent(MouseEvent {
            event_type: MouseEventType::Drag(DragEventType::Dragging(vec)),
            ..
        }) = event
        {
            let scaled_vec = *vec / scale;
            transform.translation -= scaled_vec.extend(0.)
        }
    });
    Ok(())
}

fn temp_toggle_view(
    In(trigger): In<RuntimeTrigger>,
    mut previous: ResMut<OriginalTool>,
    mut now: ResMut<Tool>,
) {
    // debug!("{trigger:?}, {previous:?}, {now:?}");
    if previous.is_none() && trigger.is_pressed() {
        previous.0 = Some(Tool::Pencil);
        now.set(Tool::View);
    } else if trigger.is_released() {
        previous.0 = None;
        now.set(Tool::Pencil);
    }
}

struct PencilToolEditData {
    line_idx: usize,
    point_idx: usize,
}
fn handle_discard(
    discard_events: &mut EventReader<DiscardPreeditEvent>,
    current_edit: &mut Option<PencilToolEditData>,
    history: &mut ChartEditHistory,
    chart: &mut Mut<rizlium_chart::chart::Chart>,
) {
    if !discard_events.is_empty() {
        discard_events.clear();
        *current_edit = None;
        history.discard_preedit(&mut **chart).unwrap();
    }
}

fn handle_editing_state(
    event: &WorldMouseEvent,
    line_idx: usize,
    point_idx: usize,
    history: &mut ChartEditHistory,
    chart: &mut Mut<rizlium_chart::chart::Chart>,
    to_game: &WorldToGame,
    world_config: &WorldViewConfig,
    pencil_config: &PencilToolConfig,
    snapping_config: &SnappingConfig,
    current_edit: &mut Option<PencilToolEditData>,
    toast: &mut ToastsStorage,
) {
    let mouse_event = &event.event;
    let snapping_context = SnappingContext::new(snapping_config, Some(&**chart));
    let Some(raw_time) = to_game.time_at_y(mouse_event.pos.y, world_config.canvas_index) else {
        toast.error(t!("edit.world_view.pencil_tool.out_of_bounds"));
        return;
    };

    let Some(raw_value) = chart
        .canvases
        .get(world_config.canvas_index)
        .map(|c| Some(mouse_event.pos.x - c.x_pos.value_padding(to_game.time.as_ref()?.0)?))
        .flatten()
    else {
        toast.error(t!("edit.world_view.pencil_tool.empty_x_pos"));
        return;
    };
    let (snapped_time, snapped_value) = snapping_context.snap_point(raw_time, raw_value);

    if matches!(mouse_event.event_type, MouseEventType::Click(_)) {
        history.submit_preedit_squash("Edit Point");
        // 已经编辑时, 点击可进行下一个的编辑
        history
            .push_preedit(
                InsertPoint {
                    line_path: line_idx.into(),
                    point_idx: None,
                    point: KeyPoint {
                        time: snapped_time,
                        value: snapped_value,
                        ease_type: pencil_config.easing,
                        relevant: LinePointData {
                            canvas: world_config.canvas_index,
                            color: color32_to_colorrgba(pencil_config.pen_color),
                        },
                    },
                },
                &mut **chart,
            )
            .unwrap();
        history.push_preedit(Nop, &mut **chart).unwrap();
        *current_edit = Some(PencilToolEditData {
            line_idx,
            point_idx: chart.lines[line_idx].points.len() - 1,
        })
    } else {
        let current_point_edit = EditPoint {
            line_path: line_idx.into(),
            point_idx,
            new_time: Some(snapped_time),
            new_x: Some(snapped_value),
            new_canvas: Some(world_config.canvas_index),
            new_color: Some(color32_to_colorrgba(pencil_config.pen_color)),
            new_easing: Some(pencil_config.easing),
        };

        if point_idx > 0 {
            let prev_point_edit = EditPoint {
                line_path: line_idx.into(),
                point_idx: point_idx - 1,
                new_time: None,
                new_x: None,
                new_canvas: None,
                new_color: None,
                new_easing: Some(pencil_config.easing),
            };
            history
                .replace_last_preedit(
                    CommandSequence {
                        commands: vec![prev_point_edit.into(), current_point_edit.into()],
                        description: "Edit Point".into(),
                    },
                    &mut **chart,
                )
                .unwrap();
        } else {
            history
                .replace_last_preedit(current_point_edit, &mut **chart)
                .unwrap();
        }
    }
}

fn handle_new_line_creation(
    event: &WorldMouseEvent,
    pencil_config: &PencilToolConfig,
    world_config: &WorldViewConfig,
    snapping_config: &SnappingConfig,
    to_game: &WorldToGame,
    history: &mut ChartEditHistory,
    chart: &mut Mut<rizlium_chart::chart::Chart>,
    toast: &mut ToastsStorage,
    current_edit: &mut Option<PencilToolEditData>,
) {
    let mouse_event = &event.event;
    let point = get_point(
        mouse_event.pos,
        pencil_config,
        world_config,
        snapping_config,
        to_game,
        chart,
        toast,
    );
    if let Some(point) = point {
        history
            .push_preedit(
                InsertLine {
                    line: Line::from_iter(vec![point; 2]),
                    at: None,
                },
                &mut **chart,
            )
            .unwrap();
        history.push_preedit(Nop, &mut **chart).unwrap();
        *current_edit = Some(PencilToolEditData {
            line_idx: chart.lines.len() - 1,
            point_idx: 1,
        })
    } else if chart.canvases.get(world_config.canvas_index).is_some() {
        toast.error(t!("edit.world_view.pencil_tool.unsupported_canvas"));
    } else {
        toast.error(t!("edit.world_view.pencil_tool.out_of_bounds"));
    }
}

fn handle_resume_editing(
    event: &WorldMouseEvent,
    entities: &Query<(Entity, &PointIndicatorId)>,
    current_edit: &mut Option<PencilToolEditData>,
) {
    if let Some(entity) = event.casted_entity {
        if let Some(entity) = entities.iter().find(|e| e.0 == entity).map(|e| e.1) {
            debug!("clicking on points");
            *current_edit = Some(PencilToolEditData {
                line_idx: entity.line_idx,
                point_idx: entity.keypoint_idx,
            });
        }
    }
}

fn pencil_tool(
    mut mouse_events: EventReader<WorldMouseEvent>,
    mut discard_events: EventReader<DiscardPreeditEvent>,
    tool: Res<Tool>,
    pencil_config: Res<PencilToolConfig>,
    world_config: Res<WorldViewConfig>,
    snapping_config: Res<SnappingConfig>,
    chart: Option<ResMut<ProjectState>>,
    mut history: ResMut<ChartEditHistory>,
    to_game: WorldToGame,
    mut current_edit: Local<Option<PencilToolEditData>>,
    entities: Query<(Entity, &PointIndicatorId)>,
    mut toast: ResMut<ToastsStorage>,
) {
    if *tool != Tool::Pencil || !to_game.avalible() {
        mouse_events.clear();
        discard_events.clear();
        return;
    }
    let Some(mut chart) = chart else {
        return;
    };
    let mut chart = chart.map_unchanged(|p| p.chart_mut());
    if !history.has_preedit() {
        *current_edit = None;
    }

    handle_discard(
        &mut discard_events,
        &mut *current_edit,
        &mut history,
        &mut chart,
    );

    for event in mouse_events.read() {
        if let Some(data) = current_edit.as_ref() {
            handle_editing_state(
                event,
                data.line_idx,
                data.point_idx,
                &mut history,
                &mut chart,
                &to_game,
                &world_config,
                &pencil_config,
                &snapping_config,
                &mut *current_edit,
                &mut toast,
            );
        } else if event.casted_entity.is_none()
            && matches!(event.event.event_type, MouseEventType::Click(_))
        {
            handle_new_line_creation(
                event,
                &pencil_config,
                &world_config,
                &snapping_config,
                &to_game,
                &mut history,
                &mut chart,
                &mut toast,
                &mut *current_edit,
            );
        } else if matches!(event.event.event_type, MouseEventType::Click(_)) {
            handle_resume_editing(event, &entities, &mut *current_edit);
        }
    }
}

fn color32_to_colorrgba(color: egui::Color32) -> ColorRGBA {
    ColorRGBA::new(
        color.r() as f32 / 255.,
        color.g() as f32 / 255.,
        color.b() as f32 / 255.,
        color.a() as f32 / 255.,
    )
}

fn get_point(
    pos: Vec3,
    pencil_config: &PencilToolConfig,
    world_config: &WorldViewConfig,
    snapping_config: &SnappingConfig,
    to_game: &WorldToGame,
    chart: &Mut<rizlium_chart::chart::Chart>,
    toast: &mut ToastsStorage
) -> Option<KeyPoint<f32, LinePointData>> {
    let snapping_context = SnappingContext::new(snapping_config, Some(&**chart));
    let Some(raw_time) = to_game.time_at_y(pos.y, world_config.canvas_index) else {
        toast.error(t!("edit.world_view.pencil_tool.out_of_bounds"));
        return None;
    };

    let Some(raw_value) = chart
        .canvases
        .get(world_config.canvas_index)
        .map(|c| Some(pos.x - c.x_pos.value_padding(to_game.time.as_ref()?.0)?))
        .flatten()
    else {
        toast.error(t!("edit.world_view.pencil_tool.empty_x_pos"));
        return None;
    };
    let (snapped_time, snapped_value) = snapping_context.snap_point(raw_time, raw_value);

    Some(KeyPoint {
        time: snapped_time,
        value: snapped_value,
        ease_type: pencil_config.easing,
        relevant: LinePointData {
            color: color32_to_colorrgba(pencil_config.pen_color),
            canvas: world_config.canvas_index,
        },
    })
}

fn select_tool(
    mut mouse_events: EventReader<WorldMouseEvent>,
    tool: Res<Tool>,
    to_game: WorldToGame,
    lines: Query<(Entity, &ChartLineId)>,
    mut selected_item: ResMut<SelectedItem>,
) {
    if *tool != Tool::Select || !to_game.avalible() {
        mouse_events.clear();
        return;
    }
    for event in mouse_events.read() {
        if event.event.event_type.is_click() {
            debug!("{event:?}");
            if let Some(entity) = event.casted_entity {
                let Some((_, line)) = lines.iter().find(|e| e.0 == entity) else {
                    continue;
                };
                selected_item.item = Some(ChartItem::LinePoint(LinePointPath(
                    LinePath(line.line_idx()),
                    line.keypoint_idx(),
                )))
            } else {
                selected_item.item = None
            }
        }
    }
}
