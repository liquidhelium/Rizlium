use bevy::prelude::*;
use helium_framework::prelude::*;
use rizlium_chart::{
    chart::{Chart, ColorRGBA, KeyPoint, Line, LinePointData},
    editing::commands::{CommandSequence, EditPoint, InsertLine, InsertPoint, Nop},
};
use rizlium_render::ChartProvider as _;

use crate::{
    extensions::{
        editing::{world_view::WorldViewConfig, ChartEditHistory},
    },
    project::ProjectState,
    utils::WorldToGame,
};

use super::{
    tool_configs::PencilToolConfig,
    Tool, DiscardPreeditEvent,
};
use super::super::{
    cam_response::{DragEventType, MouseEventType, WorldMouseEvent},
    snapping::{SnappingConfig, SnappingContext},
    PointIndicatorId,
};

pub struct PencilToolEditData {
    line_idx: usize,
    point_idx: usize,
    is_resumed: bool,
}

pub(super) fn pencil_tool(
    mut mouse_events: MessageReader<WorldMouseEvent>,
    mut discard_events: MessageReader<DiscardPreeditEvent>,
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
    let Some(chart) = chart else {
        return;
    };
    let chart = chart.map_unchanged(|p| p.chart_mut());
    if !history.has_preedit() {
        *current_edit = None;
    }

    let mut ctx = PencilContext {
        pencil_config: &pencil_config,
        world_config: &world_config,
        snapping_config: &snapping_config,
        chart: chart,
        history: &mut history,
        to_game: &to_game,
        current_edit: &mut current_edit,
        toast: &mut toast,
    };

    ctx.handle_discard(&mut discard_events);

    for event in mouse_events.read() {
        ctx.handle_event(event, &entities);
    }
}

struct PencilContext<'a, 'w> {
    pencil_config: &'a PencilToolConfig,
    world_config: &'a WorldViewConfig,
    snapping_config: &'a SnappingConfig,
    chart: Mut<'a, Chart>,
    history: &'a mut ChartEditHistory,
    to_game: &'a WorldToGame<'w>,
    current_edit: &'a mut Option<PencilToolEditData>,
    toast: &'a mut ToastsStorage,
}

impl<'a, 'w> PencilContext<'a, 'w> {
    fn handle_discard(&mut self, discard_events: &mut MessageReader<DiscardPreeditEvent>) {
        if !discard_events.is_empty() {
            discard_events.clear();
            *self.current_edit = None;
            self.history.discard_preedit(&mut self.chart).unwrap();
        }
    }

    fn handle_event(
        &mut self,
        event: &WorldMouseEvent,
        entities: &Query<(Entity, &PointIndicatorId)>,
    ) {
        match &event.event.event_type {
            MouseEventType::Drag(DragEventType::DragStarted) => {
                if event.casted_entity.is_some() {
                    self.handle_resume_editing(event, entities);
                }
            }
            MouseEventType::Drag(DragEventType::Dragging(_)) => {
                if let Some(data) = self.current_edit.as_ref() {
                    self.handle_editing_preview(event, data.line_idx, data.point_idx);
                }
            }
            MouseEventType::Drag(DragEventType::DragEnded) => {
                if let Some(data) = self.current_edit.as_ref() {
                    if data.is_resumed {
                        self.finish_editing();
                    }
                }
            }
            MouseEventType::Click(_) => {
                if let Some(data) = self.current_edit.as_ref() {
                    if !data.is_resumed {
                        self.handle_continue_commit(event, data.line_idx);
                    }
                } else if event.casted_entity.is_none() {
                    self.handle_new_line_creation(event);
                }
            }
            MouseEventType::Hover => {
                if let Some(data) = self.current_edit.as_ref() {
                    if !data.is_resumed {
                        self.handle_editing_preview(event, data.line_idx, data.point_idx);
                    }
                }
            }
            _ => {}
        }
    }

    fn finish_editing(&mut self) {
        self.history.submit_preedit_squash("Edit Point");
        *self.current_edit = None;
    }

    fn handle_continue_commit(&mut self, event: &WorldMouseEvent, line_idx: usize) {
        let point = self.get_point(event.event.pos);
        if let Some(point) = point {
            self.history.submit_preedit_squash("Edit Point");
            self.history
                .push_preedit(
                    InsertPoint {
                        line_path: line_idx.into(),
                        point_idx: None,
                        point,
                    },
                    &mut self.chart,
                )
                .unwrap();
            self.history.push_preedit(Nop, &mut self.chart).unwrap();
            *self.current_edit = Some(PencilToolEditData {
                line_idx,
                point_idx: self.chart.lines[line_idx].points.len() - 1,
                is_resumed: false,
            })
        }
    }

    fn handle_editing_preview(
        &mut self,
        event: &WorldMouseEvent,
        line_idx: usize,
        point_idx: usize,
    ) {
        let point = self.get_point(event.event.pos);
        let Some(point) = point else { return };

        let current_point_edit = EditPoint {
            line_path: line_idx.into(),
            point_idx,
            new_time: Some(point.time),
            new_x: Some(point.value),
            new_canvas: Some(point.relevant.canvas),
            new_color: Some(point.relevant.color),
            new_easing: Some(point.ease_type),
        };

        if point_idx > 0 {
            let prev_point_edit = EditPoint {
                line_path: line_idx.into(),
                point_idx: point_idx - 1,
                new_time: None,
                new_x: None,
                new_canvas: None,
                new_color: None,
                new_easing: Some(self.pencil_config.easing),
            };
            self.history
                .replace_last_preedit(
                    CommandSequence {
                        commands: vec![prev_point_edit.into(), current_point_edit.into()],
                        description: "Edit Point".into(),
                    },
                    &mut self.chart,
                )
                .unwrap();
        } else {
            self.history
                .replace_last_preedit(current_point_edit, &mut self.chart)
                .unwrap();
        }
    }

    fn handle_new_line_creation(&mut self, event: &WorldMouseEvent) {
        let mouse_event = &event.event;
        let point = self.get_point(mouse_event.pos);
        if let Some(point) = point {
            self.history
                .push_preedit(
                    InsertLine {
                        line: Line::from_iter(vec![point; 2]),
                        at: None,
                    },
                    &mut self.chart,
                )
                .unwrap();
            self.history.push_preedit(Nop, &mut self.chart).unwrap();
            *self.current_edit = Some(PencilToolEditData {
                line_idx: self.chart.lines.len() - 1,
                point_idx: 1,
                is_resumed: false,
            })
        } else if self.chart.canvases.get(self.world_config.canvas_index).is_some() {
            self.toast
                .error(t!("edit-world-view-pencil-tool-unsupported-canvas"));
        } else {
            self.toast
                .error(t!("edit-world-view-pencil-tool-out-of-bounds"));
        }
    }

    fn handle_resume_editing(
        &mut self,
        event: &WorldMouseEvent,
        entities: &Query<(Entity, &PointIndicatorId)>,
    ) {
        if let Some(entity) = event.casted_entity {
            if let Some(entity) = entities.iter().find(|e| e.0 == entity).map(|e| e.1) {
                debug!("clicking on points");
                self.history.push_preedit(Nop, &mut self.chart).unwrap();
                *self.current_edit = Some(PencilToolEditData {
                    line_idx: entity.line_idx,
                    point_idx: entity.keypoint_idx,
                    is_resumed: true,
                });
            }
        }
    }

    fn get_point(
        &mut self,
        pos: Vec3,
    ) -> Option<KeyPoint<f32, LinePointData>> {
        let snapping_context = SnappingContext::new(self.snapping_config, Some(&*self.chart));
        let Some(raw_time) = self
            .to_game
            .time_at_y(pos.y, self.world_config.canvas_index)
        else {
            self.toast
                .error(t!("edit-world-view-pencil-tool-out-of-bounds"));
            return None;
        };

        let Some(raw_value) = self
            .chart
            .canvases
            .get(self.world_config.canvas_index)
            .and_then(|c| {
                let padding = c.x_pos.value_padding(self.to_game.time.as_ref()?.0)?;
                Some(pos.x - padding)
            })
        else {
            self.toast
                .error(t!("edit-world-view-pencil-tool-empty-x-pos"));
            return None;
        };
        let (snapped_time, snapped_value) = snapping_context.snap_point(raw_time, raw_value);

        Some(KeyPoint {
            time: snapped_time,
            value: snapped_value,
            ease_type: self.pencil_config.easing,
            relevant: LinePointData {
                color: color32_to_colorrgba(self.pencil_config.pen_color),
                canvas: self.world_config.canvas_index,
            },
        })
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
