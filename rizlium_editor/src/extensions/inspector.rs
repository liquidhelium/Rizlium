use crate::extensions::editing::spline::{SplineEditorAdapter, SplineListEditor};
use std::borrow::Cow;
use bevy::{prelude::*, render::view::VisibleEntities};
use egui::{ScrollArea, Ui};
use rizlium_chart::{
    chart::{Chart, KeyPoint, Spline},
    editing::{
        chart_path::{
            BpmPath, CamMovePath, CamScalePath, CanvasPath, ChartPath, GlobalSplinePath, LinePath,
            LinePointPath, ThemeControlPath,
        },
        commands::{EditGlobalPoint, EditPoint, InsertGlobalPoint, RemoveGlobalPoint},
        ChartCommands, EditHistory, NotePath,
    },
    prelude::Tween,
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
    ThemeControl,
    BpmControl,
    CameraControl,
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
                    "Edit Point Easing",
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
                    "Edit Point Time",
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
                    "Edit Point Canvas",
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
                    "Edit Note Time",
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
        ChartItem::ThemeControl => {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for data in chart.chart().themes.iter() {
                        ui.horizontal(|ui| {
                            ui.label("Background Color:");
                            ui.color_edit_button_rgba_unmultiplied(&mut [
                                data.color.background.r,
                                data.color.background.g,
                                data.color.background.b,
                                data.color.background.a,
                            ]);
                            ui.label("FX Color:");
                            ui.color_edit_button_rgba_unmultiplied(&mut [
                                data.color.fx.r,
                                data.color.fx.g,
                                data.color.fx.b,
                                data.color.fx.a,
                            ]);
                            ui.label("Note Color:");
                            ui.color_edit_button_rgba_unmultiplied(&mut [
                                data.color.note.r,
                                data.color.note.g,
                                data.color.note.b,
                                data.color.note.a,
                            ]);
                        });
                    }
                });
            ui.separator();
            SplineListEditor::new(ThemeControlAdapter).show(
                ui,
                chart.reborrow().map_unchanged(|c| c.chart_mut()),
                &mut chart_edit_history,
            );
        }
        ChartItem::BpmControl => {
            SplineListEditor::new(BpmControlAdapter).show(
                ui,
                chart.reborrow().map_unchanged(|c| c.chart_mut()),
                &mut chart_edit_history,
            );
        }
        ChartItem::CameraControl => {
            ui.heading("Camera Scale");
            SplineListEditor::new(CamScaleAdapter).show(
                ui,
                chart.reborrow().map_unchanged(|c| c.chart_mut()),
                &mut chart_edit_history,
            );

            ui.separator();
            ui.heading("Camera Move");
            SplineListEditor::new(CamMoveAdapter).show(
                ui,
                chart.reborrow().map_unchanged(|c| c.chart_mut()),
                &mut chart_edit_history,
            );
        }
    }
}

struct ThemeControlAdapter;
impl SplineEditorAdapter for ThemeControlAdapter {
    type Tween = usize;
    type Relevant = ();
    type Path = ThemeControlPath;

    fn get_spline<'a>(&self, chart: &'a Chart) -> &'a Spline<Self::Tween, Self::Relevant> {
        &chart.theme_control
    }
    fn path(&self, index: usize) -> Self::Path {
        ThemeControlPath::new(index)
    }
    fn edit_command(
        &self,
        path: Self::Path,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::EditThemePoint(EditGlobalPoint {
            path,
            new_time: Some(point.time),
            new_value: Some(point.value),
            new_easing: Some(point.ease_type),
        })
    }
    fn add_command(
        &self,
        index: usize,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::InsertThemePoint(InsertGlobalPoint::new(point, Some(index)))
    }
    fn remove_command(&self, index: usize) -> ChartCommands {
        ChartCommands::RemoveThemePoint(RemoveGlobalPoint {
            path: GlobalSplinePath::new(index),
        })
    }
    fn value_ui(&self, ui: &mut Ui, value: &mut Self::Tween) -> egui::Response {
        ui.add(egui::DragValue::new(value))
    }
}

struct BpmControlAdapter;
impl SplineEditorAdapter for BpmControlAdapter {
    type Tween = f32;
    type Relevant = ();
    type Path = BpmPath;

    fn get_spline<'a>(&self, chart: &'a Chart) -> &'a Spline<Self::Tween, Self::Relevant> {
        &chart.bpm
    }
    fn path(&self, index: usize) -> Self::Path {
        BpmPath::new(index)
    }
    fn edit_command(
        &self,
        path: Self::Path,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::EditBpmPoint(EditGlobalPoint {
            path,
            new_time: Some(point.time),
            new_value: Some(point.value),
            new_easing: Some(point.ease_type),
        })
    }
    fn add_command(
        &self,
        index: usize,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::InsertBpmPoint(InsertGlobalPoint::new(point, Some(index)))
    }
    fn remove_command(&self, index: usize) -> ChartCommands {
        ChartCommands::RemoveBpmPoint(RemoveGlobalPoint {
            path: GlobalSplinePath::new(index),
        })
    }
    fn value_ui(&self, ui: &mut Ui, value: &mut Self::Tween) -> egui::Response {
        ui.add(egui::DragValue::new(value))
    }
}

struct CamScaleAdapter;
impl SplineEditorAdapter for CamScaleAdapter {
    type Tween = f32;
    type Relevant = ();
    type Path = CamScalePath;

    fn get_spline<'a>(&self, chart: &'a Chart) -> &'a Spline<Self::Tween, Self::Relevant> {
        &chart.cam_scale
    }
    fn path(&self, index: usize) -> Self::Path {
        CamScalePath::new(index)
    }
    fn edit_command(
        &self,
        path: Self::Path,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::EditCamScalePoint(EditGlobalPoint {
            path,
            new_time: Some(point.time),
            new_value: Some(point.value),
            new_easing: Some(point.ease_type),
        })
    }
    fn add_command(
        &self,
        index: usize,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::InsertCamScalePoint(InsertGlobalPoint::new(point, Some(index)))
    }
    fn remove_command(&self, index: usize) -> ChartCommands {
        ChartCommands::RemoveCamScalePoint(RemoveGlobalPoint {
            path: GlobalSplinePath::new(index),
        })
    }
    fn value_ui(&self, ui: &mut Ui, value: &mut Self::Tween) -> egui::Response {
        ui.add(egui::DragValue::new(value).speed(0.01))
    }
}

struct CamMoveAdapter;
impl SplineEditorAdapter for CamMoveAdapter {
    type Tween = f32;
    type Relevant = ();
    type Path = CamMovePath;

    fn get_spline<'a>(&self, chart: &'a Chart) -> &'a Spline<Self::Tween, Self::Relevant> {
        &chart.cam_move
    }
    fn path(&self, index: usize) -> Self::Path {
        CamMovePath::new(index)
    }
    fn edit_command(
        &self,
        path: Self::Path,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::EditCamMovePoint(EditGlobalPoint {
            path,
            new_time: Some(point.time),
            new_value: Some(point.value),
            new_easing: Some(point.ease_type),
        })
    }
    fn add_command(
        &self,
        index: usize,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands {
        ChartCommands::InsertCamMovePoint(InsertGlobalPoint::new(point, Some(index)))
    }
    fn remove_command(&self, index: usize) -> ChartCommands {
        ChartCommands::RemoveCamMovePoint(RemoveGlobalPoint {
            path: GlobalSplinePath::new(index),
        })
    }
    fn value_ui(&self, ui: &mut Ui, value: &mut Self::Tween) -> egui::Response {
        ui.add(egui::DragValue::new(value).speed(0.01))
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
    description: impl Into<Cow<'static, str>>,
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
        history.submit_preedit_squash(description);
    }
}

fn debug_window(
    InMut(ui): InMut<Ui>,
    history: Res<ChartEditHistory>,
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
            ui.heading("Preedits");
            for ed in history.preedit_datas() {
                ui.label(format!("{:#?}", ed.inverse()));
            }
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
