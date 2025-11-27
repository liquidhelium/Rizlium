use rust_i18n::t;
use strum::EnumIter;

use bevy::{input::mouse::MouseWheel, prelude::*};
use egui::Ui;
use rizlium_chart::editing::chart_path::{LinePath, LinePointPath};
use rizlium_render::ChartLineId;

use self::tool_configs::{PencilToolConfig, ToolConfigExt};
use crate::{
    extensions::inspector::{ChartItem, SelectedItem},
    utils::WorldToGame,
};
use helium_framework::prelude::*;

use super::{
    cam_response::{DragEventType, MouseEvent, MouseEventType, ScreenMouseEvent, WorldMouseEvent},
    edit_view_or_tool_focused,
    WorldCam,
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
                (view_tool, pencil::pencil_tool, select_tool)
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
pub mod pencil;

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
