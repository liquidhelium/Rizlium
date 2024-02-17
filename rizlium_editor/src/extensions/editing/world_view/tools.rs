use bevy::{input::mouse::MouseWheel, math::vec2, prelude::*};
use rizlium_chart::{
    chart::Line,
    editing::commands::InsertLine,
};
use rizlium_render::GameChart;

use crate::{
    extensions::editing::ChartEditHistory, hotkeys::{Hotkey, HotkeysExt, RuntimeTrigger, TriggerType}, utils::WorldToGame, ActionsExt
};

use super::{
    cam_response::{DragEventType, MouseEvent, MouseEventType, ScreenMouseEvent, WorldMouseEvent},
    edit_view_focused, WorldCam,
};

pub fn is_tool(tool: Tool) -> impl Condition<()> {
    edit_view_focused().and_then(resource_exists_and_equals(tool))
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
            .add_systems(Update, (view_tool, pencil_tool));
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
            [Hotkey::new([KeyCode::P], edit_view_focused())],
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

#[derive(Resource, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub enum Tool {
    #[default]
    View,
    Pencil,
    Select,
}

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

fn pencil_tool(
    mut events: EventReader<WorldMouseEvent>,
    tool: Res<Tool>,
    chart: Option<ResMut<GameChart>>,
    mut history: ResMut<ChartEditHistory>,
    to_game: WorldToGame
) {
    if *tool != Tool::Pencil || !to_game.avalible() {
        events.clear();
        return;
    }
    let Some(mut chart) = chart else {
        return;
    };
    for event in events.read() {
        if !event.casted_on_entity && matches!(event.event.event_type, MouseEventType::Click(_)) {
            let event = &event.event;
            history
                .push(
                    InsertLine {
                        line: Line::new_two_points(
                            map_to_game(&to_game, event.pos.xy(), 0),
                            map_to_game(&to_game, event.pos.xy() + vec2(0., 60.), 0),
                        ),
                        at: None,
                    },
                    &mut chart,
                )
                .unwrap();
        }
    }
}

fn map_to_game(to_game: &WorldToGame, pos: Vec2, canvas: usize) -> [f32; 2] {
    [
        to_game.time_at_y(pos.y, canvas).unwrap(),
        pos.x,
    ] // time, value
}
