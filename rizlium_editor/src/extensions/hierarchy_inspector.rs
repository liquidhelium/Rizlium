use bevy::prelude::*;
use helium_framework::prelude::{Actions, TabRegistrationExt as _};
use rizlium_chart::{
    chart::KeyPoint,
    editing::{
        chart_path::CanvasPath,
        commands::{
            InsertCanvas, InsertLine, RemoveCanvas, RemoveLine, RenameCanvas, RenameLine,
        },
        ChartCommands,
    },
};
use rizlium_render::ChartProvider as _;
use crate::t;

use crate::extensions::inspector::{ChartItem, SelectedItem};

pub struct HierarchyInspector;
impl bevy::prelude::Plugin for HierarchyInspector {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_tab(
            "hierarchy",
            t!("hierarchy-tab"),
            hierarchy_ui,
            crate::project::ProjectState::has_chart_system(),
        );
    }
}

#[derive(Default)]
struct HierarchyState {
    editing_item: Option<ChartItem>,
    editing_text: String,
    has_pushed_preedit: bool,
}

fn hierarchy_ui(
    InMut(mut ui): InMut<egui::Ui>,
    chart: Res<crate::project::ProjectState>,
    mut select: ResMut<SelectedItem>,
    mut actions: Actions,
    mut state: Local<HierarchyState>,
) {
    // chart must exist.
    let chart = chart.chart();
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show_viewport(ui, |ui, _rect| {
            if ui
                .selectable_label(
                    matches!(select.item, Some(ChartItem::BpmControl)),
                    "BPM Control Points",
                )
                .clicked()
            {
                // select BPM control points
                select.item = Some(ChartItem::BpmControl);
            }
            if ui
                .selectable_label(
                    matches!(select.item, Some(ChartItem::CameraControl)),
                    "Camera Control Points",
                )
                .clicked()
            {
                // select camera control points
                select.item = Some(ChartItem::CameraControl);
            }
            if ui
                .selectable_label(
                    matches!(select.item, Some(ChartItem::ThemeControl)),
                    "Theme Control Data",
                )
                .clicked()
            {
                // select theme control data
                select.item = Some(ChartItem::ThemeControl);
            }
            ui.separator();
            ui.collapsing("Canvases", |ui| {
                for (i, canvas) in chart.canvases.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let is_editing = matches!(state.editing_item, Some(ChartItem::Canvas(ref p)) if p.0 == i);
                        if is_editing {
                            let response = ui.text_edit_singleline(&mut state.editing_text);
                            if response.changed() {
                                let command = ChartCommands::RenameCanvas(RenameCanvas {
                                    canvas_path: CanvasPath(i),
                                    name: state.editing_text.clone(),
                                });
                                if state.has_pushed_preedit {
                                    actions.queue_action(
                                        &"edit.replace_last_preedit".into(),
                                        In(command),
                                    );
                                } else {
                                    actions.queue_action(
                                        &"edit.push_preedit".into(),
                                        In(command),
                                    );
                                    state.has_pushed_preedit = true;
                                }
                            }
                            if response.lost_focus()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                actions.queue_action(&"edit.submit_preedit".into(), In(()));
                                state.editing_item = None;
                                state.has_pushed_preedit = false;
                            }
                        } else {
                            let response = ui.selectable_label(
                                matches!(select.item, Some(ChartItem::Canvas(ref p)) if p.0 == i),
                                &canvas.name,
                            );
                            if response.clicked() {
                                // select this canvas
                                select.item = Some(ChartItem::Canvas(CanvasPath(i)));
                            }
                            if response.double_clicked() {
                                state.editing_item = Some(ChartItem::Canvas(CanvasPath(i)));
                                state.editing_text = canvas.name.clone();
                                state.has_pushed_preedit = false;
                            }
                        }

                        if ui.button("x").clicked() {
                            // delete this canvas
                            select.item = None;
                            actions.queue_action(
                                &"edit.push_edit_command".into(),
                                In(ChartCommands::RemoveCanvas(RemoveCanvas {
                                    canvas_path: CanvasPath(i),
                                })),
                            );
                        }
                    });
                }
                if ui
                    .vertical_centered_justified(|ui| ui.add(egui::Button::new("Add Canvas")))
                    .inner
                    .clicked()
                {
                    actions.queue_action(
                        &"edit.push_edit_command".into(),
                        In(ChartCommands::InsertCanvas(InsertCanvas {
                            canvas: rizlium_chart::prelude::Canvas {
                                name: format!("Canvas {}", chart.canvases.len() + 1),
                                x_pos: vec![KeyPoint {
                                    time: 0.0,
                                    value: 0.0,
                                    ease_type: rizlium_chart::prelude::EasingId::Linear,
                                    relevant: (),
                                }]
                                .into(),
                                speed: vec![KeyPoint {
                                    time: 0.0,
                                    value: 1.0,
                                    ease_type: rizlium_chart::prelude::EasingId::Linear,
                                    relevant: (),
                                }]
                                .into(),
                            },
                            at: None,
                        })),
                    );
                }
            });
            ui.collapsing("Lines", |ui| {
                for (i, line) in chart.lines.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let is_editing = matches!(state.editing_item, Some(ChartItem::Line(ref p)) if p.0 == i);
                        if is_editing {
                            let response = ui.text_edit_singleline(&mut state.editing_text);
                            if response.changed() {
                                let command = ChartCommands::RenameLine(RenameLine {
                                    line_path: rizlium_chart::editing::chart_path::LinePath(i),
                                    name: state.editing_text.clone(),
                                });
                                if state.has_pushed_preedit {
                                    actions.queue_action(
                                        &"edit.replace_last_preedit".into(),
                                        In(command),
                                    );
                                } else {
                                    actions.queue_action(
                                        &"edit.push_preedit".into(),
                                        In(command),
                                    );
                                    state.has_pushed_preedit = true;
                                }
                            }
                            if response.lost_focus()
                                || ui.input(|i| i.key_pressed(egui::Key::Enter))
                            {
                                actions.queue_action(&"edit.submit_preedit".into(), ());
                                state.editing_item = None;
                                state.has_pushed_preedit = false;
                            }
                        } else {
                            let response = ui.selectable_label(
                                matches!(select.item, Some(ChartItem::Line(ref p)) if p.0 == i),
                                &line.name,
                            );
                            if response.clicked() {
                                // select this line
                                select.item = Some(ChartItem::Line(
                                    rizlium_chart::editing::chart_path::LinePath(i),
                                ));
                            }
                            if response.double_clicked() {
                                state.editing_item = Some(ChartItem::Line(
                                    rizlium_chart::editing::chart_path::LinePath(i),
                                ));
                                state.editing_text = line.name.clone();
                                state.has_pushed_preedit = false;
                            }
                        }

                        if ui.button("x").clicked() {
                            // delete this line
                            select.item = None;
                            actions.queue_action(
                                &"edit.push_edit_command".into(),
                                In(ChartCommands::RemoveLine(RemoveLine {
                                    line_path: rizlium_chart::editing::chart_path::LinePath(i),
                                })),
                            );
                        }
                    });
                }
            });
        });
}
