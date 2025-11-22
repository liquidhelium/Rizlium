use bevy::{prelude::*, render::view::VisibleEntities};
use egui::{ScrollArea, Ui};
use rizlium_chart::{
    chart::Chart,
    editing::{
        chart_path::{CanvasPath, ChartPath, LinePath, LinePointPath},
        commands::EditPoint,
        ChartCommands, EditHistory, NotePath,
    },
};
use rizlium_render::{ChartProvider, GameCamera};
use rust_i18n::t;

use helium_framework::prelude::*;

use crate::{project::ProjectState, widgets::enum_selector, RizliumDockStateMirror};

use super::editing::ChartEditHistory;

#[derive(Resource, Default)]
pub struct SelectedItem {
    pub item: Option<ChartItem>,
}

pub enum ChartItem {
    LinePoint(LinePointPath),
    Line(LinePath),
    Note(NotePath),
    Canvas(CanvasPath),
}

pub struct Inspector;

impl Plugin for Inspector {
    fn build(&self, app: &mut App) {
        app.register_tab(
            "inspector",
            t!("inspector.tab"),
            logs,
            ProjectState::has_chart_system(),
        )
        .init_resource::<SelectedItem>();
        app.register_tab(
            "debugger",
            t!("debugger.tab"),
            debug_window,
            resource_exists::<ChartEditHistory>,
        );
    }
}

fn logs(
    InMut(mut ui): InMut<Ui>,
    mut chart: ResMut<ProjectState>,
    selected: Res<SelectedItem>,
    mut chart_edit_history: ResMut<ChartEditHistory>,
) {
    let Some(ref item) = selected.item else {
        ui.weak(t!("tab.logs.select_to_inspect"));
        return;
    };
    let ui = &mut ui;
    match item {
        ChartItem::LinePoint(l) => {
            ui.columns(2, |columns| {
                columns[0].label("easing:");
                edit_scope(
                    &mut columns[1],
                    *l,
                    chart.reborrow().map_unchanged(|chart| chart.chart_mut()),
                    &mut chart_edit_history,
                    |ui, easing| enum_selector(&mut easing.ease_type, ui),
                    |path, value| {
                        ChartCommands::EditPoint(EditPoint {
                            line_path: path.0,
                            point_idx: path.1,
                            new_easing: Some(value.ease_type),
                            ..Default::default()
                        })
                    },
                );
                columns[0].label("time:");
                edit_scope(
                    &mut columns[1],
                    *l,
                    chart.reborrow().map_unchanged(|chart| chart.chart_mut()),
                    &mut chart_edit_history,
                    |ui, point| ui.add(egui::DragValue::new(&mut point.time).speed(0.01)),
                    |path, value| {
                        ChartCommands::EditPoint(EditPoint {
                            line_path: path.0,
                            point_idx: path.1,
                            new_time: Some(value.time),
                            ..Default::default()
                        })
                    },
                );
                columns[0].label("canvas:");
                edit_scope(
                    &mut columns[1],
                    *l,
                    chart.reborrow().map_unchanged(|chart| chart.chart_mut()),
                    &mut chart_edit_history,
                    |ui, point| ui.add(egui::DragValue::new(&mut point.relevant.canvas).speed(1)),
                    |path, value| {
                        ChartCommands::EditPoint(EditPoint {
                            line_path: path.0,
                            point_idx: path.1,
                            new_canvas: Some(value.relevant.canvas),
                            ..Default::default()
                        })
                    },
                );
            });
        }
        ChartItem::Line(l) => {
            show_ui(ui, *l, &chart.chart(), |ui, line| {
                ui.strong(format!("Line {}:", l.0));
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show_viewport(ui, |ui, _rect| {
                        for (i, point) in line.points.iter().enumerate() {
                            ui.label(format!(
                                "Point {}: time = {:.2}, value = {:.2}",
                                i, point.time, point.value
                            ));
                        }
                    });
            });
        }
        ChartItem::Note(n) => {
            // show_ui(ui, *n, &chart.chart(), |ui, note| {
            ui.strong(format!("Line {} Note {}:", n.0 .0, n.1));
            ui.columns(2, |columns| {
                columns[0].label("time:");
                edit_scope(
                    &mut columns[1],
                    *n,
                    chart.reborrow().map_unchanged(|chart| chart.chart_mut()),
                    &mut chart_edit_history,
                    |ui, note| ui.add(egui::DragValue::new(&mut note.time).speed(0.01)),
                    |path, value| {
                        ChartCommands::ChangeNoteTime(
                            rizlium_chart::editing::commands::ChangeNoteTime {
                                note_path: path,
                                modify_to: value.time,
                            },
                        )
                    },
                );
            });
            // });
        }
        ChartItem::Canvas(c) => {
            show_ui(ui, *c, &chart.chart(), |ui, canvas| {
                ui.strong(format!("Canvas {}:", c.0));
                egui::ScrollArea::vertical()
                    .auto_shrink([false; 2])
                    .show_viewport(ui, |ui, _rect| {
                        for (i, point) in canvas.x_pos.iter().enumerate() {
                            ui.label(format!(
                                "X Position Point {}: time = {:.2}, value = {:.2}",
                                i, point.time, point.value
                            ));
                        }
                        for (i, point) in canvas.speed.iter().enumerate() {
                            ui.label(format!(
                                "Speed Point {}: time = {:.2}, value = {:.2}",
                                i, point.time, point.value
                            ));
                        }
                    });
            });
        }
    }
}

fn show_ui<P: ChartPath>(
    ui: &mut Ui,
    item_path: P,
    chart: &Chart,
    show: impl FnOnce(&mut Ui, &P::Out),
) {
    match item_path.get(chart) {
        Ok(item) => show(ui, item),
        Err(err) => {
            ui.colored_label(egui::Color32::RED, err.to_string());
        }
    };
}

pub fn edit_scope<P, T, F, C>(
    ui: &mut Ui,
    path: P,
    mut chart: Mut<Chart>,
    history: &mut EditHistory,
    draw_ui: F,
    to_command: C,
) where
    P: ChartPath<Out = T>,
    T: Clone,
    F: FnOnce(&mut Ui, &mut T) -> egui::Response,
    C: FnOnce(P, T) -> ChartCommands,
{
    // 1. 安全获取当前值
    let Ok(original) = path.get(&chart) else {
        ui.add_enabled_ui(false, |ui| ui.label("Invalid Path")); // 路径失效时的处理
        return;
    };
    let mut value = original.clone();

    // 2. 绘制 UI
    let response = draw_ui(ui, &mut value);

    // 3. 处理逻辑
    if response.changed() {
        let command = to_command(path, value);

        // 判断是 "新操作" 还是 "更新操作"
        // drag_started: 明确的拖拽开始
        // !has_preedit: 可能是点击 Checkbox 或 TextEdit 的第一次输入
        if response.drag_started() || !history.has_preedit() {
            let _ = history.push_preedit(command, &mut chart);
        } else {
            let _ = history.replace_last_preedit(command, &mut chart);
        }
    }

    // 4. 提交逻辑
    // drag_released: 拖拽结束
    // lost_focus: 输入框失去焦点 (回车或点击别处)
    if response.drag_stopped() || response.lost_focus() {
        history.submit_preedit_squash();
    }
}

fn debug_window(
    InMut(ui): InMut<Ui>,
    // history: Res<ChartEditHistory>,
    // mut event: EventReader<WorldMouseEvent>,
    mirror: Res<RizliumDockStateMirror>,
    camera: Query<&VisibleEntities, With<GameCamera>>,
) {
    ScrollArea::vertical()
        .auto_shrink(false)
        .show(ui, |ui| -> Result<()> {
            // ui.heading("cast_result");
            // ui.label(format!("{:?}", event.read().next()));
            // for it in history.history_descriptions() {
            //     ui.label(it.clone());
            // }
            // ui.heading("Preedits");
            // for ed in history.preedit_datas() {
            //     ui.label(format!("{:#?}", ed.inverse()));
            // }
            let cam = camera.single()?;
            let sorted_entities: Vec<_> = cam
                .entities
                .iter()
                .map(|e| {
                    let mut vec = e.1.clone();
                    vec.sort();
                    vec
                })
                .collect();
            ui.code_editor(&mut format!("{sorted_entities:?}"));
            Ok(())
        })
        .inner
        .unwrap();
}
