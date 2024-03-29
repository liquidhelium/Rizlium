use strum::EnumIter;

use bevy::{input::mouse::MouseWheel, math::vec2, prelude::*};
use egui::Ui;
use rizlium_chart::{
    chart::{ColorRGBA, EasingId, KeyPoint, Line, LinePointData},
    editing::commands::{EditPoint, InsertLine, InsertPoint, Nop},
};
use rizlium_render::GameChart;

use crate::{
    extensions::editing::ChartEditHistory,
    hotkeys::{Hotkey, HotkeysExt, RuntimeTrigger, TriggerType},
    tab_system::tab_opened,
    utils::WorldToGame,
    ActionsExt,
};

use self::tool_configs::{PencilToolConfig, ToolConfigExt};

use super::{
    cam_response::{DragEventType, MouseEvent, MouseEventType, ScreenMouseEvent, WorldMouseEvent},
    edit_view_or_tool_focused, WorldCam,
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
            .init_tool_config::<tool_configs::PencilToolConfig>()
            .add_systems(
                Update,
                (view_tool, pencil_tool).run_if(tab_opened("edit.world_view")),
            );
        app.register_action(
            "edit.world_view.temp_toggle_view",
            "Temporarily switch to tool View.",
            temp_toggle_view,
        );
        app.register_action(
            "edit.world_view.to_pencil",
            "Switch to tool Pencil.",
            switch_tool(Tool::Pencil),
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
    }
}

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug, EnumIter)]
pub enum Tool {
    #[default]
    View,
    Pencil,
    Select,
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
    mut events: EventReader<WorldMouseEvent>,
    tool: Res<Tool>,
    pencil_config: Res<PencilToolConfig>,
    chart: Option<ResMut<GameChart>>,
    mut history: ResMut<ChartEditHistory>,
    to_game: WorldToGame,
    mut current_edit: Local<Option<PencilToolEditData>>,
) {
    if *tool != Tool::Pencil || !to_game.avalible() {
        events.clear();
        return;
    }
    let Some(mut chart) = chart else {
        return;
    };
    for event in events.read() {
        if let Some(data) = current_edit.as_ref() {
            let event = &event.event;
            if matches!(event.event_type, MouseEventType::Click(_)) {
                // 已经编辑时, 点击可进行下一个的编辑
                history.submit_preedit();
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
                            new_easing: Some(pencil_config.easing)
                        },
                        &mut chart,
                    )
                    .unwrap();
            }
        } else if !event.casted_on_entity
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
