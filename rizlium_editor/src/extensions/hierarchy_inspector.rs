use bevy::prelude::*;
use helium_framework::prelude::{Actions, TabRegistrationExt as _};
use rizlium_chart::{
    chart::KeyPoint,
    editing::{
        chart_path::CanvasPath,
        commands::{InsertCanvas, InsertLine, RemoveCanvas, RemoveLine},
        ChartCommands,
    },
};
use rizlium_render::ChartProvider as _;
use rust_i18n::t;

use crate::extensions::inspector::{ChartItem, SelectedItem};

pub struct HierarchyInspector;
impl bevy::prelude::Plugin for HierarchyInspector {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.register_tab(
            "hierarchy",
            t!("hierarchy.tab"),
            hierarchy_ui,
            crate::project::ProjectState::has_chart_system(),
        );
    }
}
fn hierarchy_ui(
    InMut(mut ui): InMut<egui::Ui>,
    chart: Res<crate::project::ProjectState>,
    mut select: ResMut<SelectedItem>,
    mut actions: Actions,
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
                        if ui
                            .selectable_label(
                                matches!(
                                    select.item,
                                    Some(ChartItem::Canvas(ref p)) if p.0 == i
                                ),
                                format!("Canvas {}", i),
                            )
                            .clicked()
                        {
                            // select this canvas
                            select.item = Some(ChartItem::Canvas(CanvasPath(i)));
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
                    if ui
                        .selectable_label(
                            matches!(
                            select.item,
                            Some(ChartItem::Line(ref p))
                            if p.0 == i
                            ),
                            format!("Line {}", i),
                        )
                        .clicked()
                    {
                        // select this line
                        select.item = Some(ChartItem::Line(
                            rizlium_chart::editing::chart_path::LinePath(i),
                        ));
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
                }
            });
        });
}
