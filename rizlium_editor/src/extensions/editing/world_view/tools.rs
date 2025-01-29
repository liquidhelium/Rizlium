use rust_i18n::t;
use strum::EnumIter;

use bevy::{input::mouse::MouseWheel, prelude::*};
use egui::Ui;
use rizlium_chart::{
    chart::{ColorRGBA, KeyPoint, Line, LinePointData},
    editing::{
        chart_path::{LinePath, LinePointPath},
        commands::{EditPoint, InsertLine, InsertPoint, Nop},
    },
};
use rizlium_render::{ChartLineId, GameChart};

use self::tool_configs::{PencilToolConfig, ToolConfigExt};
use crate::{
    extensions::{
        editing::ChartEditHistory,
        inspector::{ChartItem, SelectedItem},
    },
    utils::WorldToGame,
};
use helium_framework::prelude::*;

use super::{
    cam_response::{DragEventType, MouseEvent, MouseEventType, ScreenMouseEvent, WorldMouseEvent},
    edit_view_or_tool_focused, PointIndicatorId, WorldCam,
};

pub fn is_tool(tool: Tool) -> impl Condition<()> {
    edit_view_or_tool_focused().and_then(resource_exists_and_equals(tool))
}

pub fn previous_tool(tool: Tool) -> impl Condition<()> {
    resource_exists_and_equals(OriginalTool(Some(tool))).and_then(|| true)
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
                (view_tool, pencil_tool, select_tool).run_if(tab_opened("edit.world_view")),
            );
        app.register_action(
            "edit.world_view.temp_toggle_view",
            t!("edit.world_view.temp_toggle_view"),
            temp_toggle_view,
        );
        app.register_action(
            "edit.world_view.to_pencil",
            t!("edit.world_view.to_pencil.desc"),
            switch_tool(Tool::Pencil),
        );
        app.register_action(
            "edit.world_view.to_select",
            t!("edit.world_view.to_select.desc"),
            switch_tool(Tool::Select),
        );
        app.register_action(
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
                is_tool(Tool::Pencil)
                    .or_else(previous_tool(Tool::Pencil).and_then(is_tool(Tool::View))),
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
    ev.send_default();
}

impl Tool {
    pub fn config_ui(&self, ui: &mut Ui, world: &mut World) {
        match self {
            Self::Pencil => tool_configs::show_window::<tool_configs::PencilToolConfig>(ui, world),
            _ => (),
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
    mut camera: Query<(&mut OrthographicProjection, &mut Transform), With<WorldCam>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    tool: Res<Tool>,
) {
    if *tool != Tool::View {
        mouse_wheel.clear();
        events.clear();
        return;
    }
    let (mut projection, mut transform) = camera.single_mut();
    let mut scale = 1. / projection.scale;
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
}

fn temp_toggle_view(
    In(trigger): In<RuntimeTrigger>,
    mut previous: ResMut<OriginalTool>,
    mut now: ResMut<Tool>,
) {
    debug!("{trigger:?}, {previous:?}, {now:?}");
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

fn pencil_tool(
    mut mouse_events: EventReader<WorldMouseEvent>,
    mut discard_events: EventReader<DiscardPreeditEvent>,
    tool: Res<Tool>,
    pencil_config: Res<PencilToolConfig>,
    chart: Option<ResMut<GameChart>>,
    mut history: ResMut<ChartEditHistory>,
    to_game: WorldToGame,
    mut current_edit: Local<Option<PencilToolEditData>>,
    mut entities: Query<(Entity, &PointIndicatorId)>,
) {
    if *tool != Tool::Pencil || !to_game.avalible() {
        mouse_events.clear();
        discard_events.clear();
        return;
    }
    let Some(mut chart) = chart else {
        return;
    };
    if !history.has_preedit() {
        *current_edit = None;
    }
    // discard key was pressed, so just discard the preedit
    if !discard_events.is_empty() {
        discard_events.clear();
        *current_edit = None;
        history.discard_preedit(&mut chart).unwrap();
        history.submit_preedit();
    }
    for event in mouse_events.read() {
        if let Some(data) = current_edit.as_ref() {
            let event = &event.event;
            if matches!(event.event_type, MouseEventType::Click(_)) {
                history.submit_preedit_squash();
                // 已经编辑时, 点击可进行下一个的编辑
                history
                    .push_preedit(
                        InsertPoint {
                            line_path: data.line_idx.into(),
                            point_idx: None,
                            point: KeyPoint {
                                time: to_game
                                    .time_at_y(event.pos.y, pencil_config.canvas)
                                    .unwrap(),
                                value: event.pos.x,
                                ease_type: pencil_config.easing,
                                relevant: LinePointData {
                                    canvas: pencil_config.canvas,
                                    color: color32_to_colorrgba(pencil_config.pen_color),
                                },
                            },
                        },
                        &mut chart,
                    )
                    .unwrap();
                history.push_preedit(Nop, &mut chart).unwrap();
                *current_edit = Some(PencilToolEditData {
                    line_idx: data.line_idx,
                    point_idx: chart.lines[data.line_idx].points.len() - 1,
                })
            } else {
                history
                    .replace_last_preedit(
                        EditPoint {
                            line_path: data.line_idx.into(),
                            point_idx: data.point_idx,
                            new_time: Some(
                                to_game
                                    .time_at_y(event.pos.y, pencil_config.canvas)
                                    .unwrap(),
                            ),
                            new_x: Some(event.pos.x),
                            new_canvas: Some(pencil_config.canvas),
                            new_color: Some(color32_to_colorrgba(pencil_config.pen_color)),
                            new_easing: Some(pencil_config.easing),
                        },
                        &mut chart,
                    )
                    .unwrap();
            }
        } else if event.casted_entity.is_none()
            && matches!(event.event.event_type, MouseEventType::Click(_))
        {
            let event = &event.event;
            history
                .push_preedit(
                    InsertLine {
                        line: Line::new_from_points(vec![
                            get_point(
                                event.pos,
                                &pencil_config,
                                &to_game,
                            );
                            2
                        ]),
                        at: None,
                    },
                    &mut chart,
                )
                .unwrap();
            history.push_preedit(Nop, &mut chart).unwrap();
            *current_edit = Some(PencilToolEditData {
                line_idx: chart.lines.len() - 1,
                point_idx: 1,
            })
        } else if matches!(event.event.event_type, MouseEventType::Click(_)) {
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
    to_game: &WorldToGame,
) -> KeyPoint<f32, LinePointData> {
    KeyPoint {
        time: to_game.time_at_y(pos.y, pencil_config.canvas).unwrap(),
        value: pos.x,
        ease_type: pencil_config.easing,
        relevant: LinePointData {
            color: color32_to_colorrgba(pencil_config.pen_color),
            canvas: pencil_config.canvas,
        },
    }
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
