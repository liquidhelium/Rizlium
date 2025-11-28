use crate::extensions::inspector::edit_scope;
use bevy::prelude::Mut;
use egui::{
    emath::RectTransform, epaint::PathShape, pos2, remap, Color32, Pos2, Rect, Response, Sense,
    Stroke, Ui,
};
use rizlium_chart::{
    chart::{invlerp, Chart, KeyPoint},
    editing::{chart_path::ChartPath, ChartCommands, EditHistory},
    prelude::{Spline, Tween},
};

pub trait TransformHelper {
    fn map_x(&self, x: f32) -> f32;
    fn map_y(&self, y: f32) -> f32;
}

impl TransformHelper for RectTransform {
    fn map_x(&self, x: f32) -> f32 {
        remap(x, self.from().x_range(), self.to().x_range())
    }
    fn map_y(&self, y: f32) -> f32 {
        remap(y, self.from().y_range(), self.to().y_range())
    }
}

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct SplineView<'a, R> {
    spline: &'a Spline<f32, R>,
    screen_area: Rect,
    view_area: Rect,
    spline_area: Rect,
    visible_spline_area: Rect,
    view_to_spline: RectTransform,
    orientation: Orientation,
}

impl<'a, R> SplineView<'a, R> {
    pub fn new(
        ui: &mut Ui,
        spline: &'a Spline<f32, R>,
        visible_spline_area: Option<Rect>,
        orientation: Orientation,
    ) -> Self {
        // Use the available rect as both screen and view area to ensure correct positioning relative to the window
        let view_area = ui.available_rect_before_wrap();
        let screen_area = view_area;

        let spline_area = {
            let rect0 = spline.rect();
            Rect::from_two_pos(rect0[0].into(), rect0[1].into())
        };
        let visible_spline_area = visible_spline_area.unwrap_or(spline_area);
        Self {
            spline,
            screen_area,
            view_area,
            spline_area,
            visible_spline_area,
            view_to_spline: RectTransform::from_to(view_area, visible_spline_area),
            orientation,
        }
    }

    pub fn ui(&self, ui: &mut Ui) -> Response {
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click());
        if self.spline.is_empty() {
            return response;
        }
        let mut circles_view = Vec::<Pos2>::new();
        let mut linepoints_view = Vec::<Pos2>::new();
        let mut current_screen_x = self.view_area.min.x;
        let mut current_t = self.view_to_spline.map_x(current_screen_x);
        let mut current_keypoint_idx = match self.spline.keypoint_at(current_t) {
            Ok(idx) => {
                let point = self.spline.points().get(idx).unwrap();
                let point_view = self
                    .view_to_spline
                    .inverse()
                    .transform_pos(pos2(point.time, point.value));
                circles_view.push(point_view);
                linepoints_view.push(point_view);
                idx
            }
            Err(idx) => {
                if idx == 0 {
                    // clamp segment_point_index to first one
                    let point = self.spline.points().get(idx).unwrap();
                    let point_view = self
                        .view_to_spline
                        .inverse()
                        .transform_pos(pos2(point.time, point.value));
                    circles_view.push(point_view);
                    linepoints_view.push(point_view);
                    current_t = point.time + 0.01;
                    current_screen_x =
                        self.view_to_spline.inverse().map_x(current_t).ceil() + 2.0;
                    idx
                } else {
                    return response;
                }
            }
        };
        loop {
            let this_point = self.spline.points().get(current_keypoint_idx).unwrap();
            let Some(next_point) = self.spline.points().get(current_keypoint_idx + 1) else {
                break;
            };
            while current_t < next_point.time {
                let value = f32::ease(
                    this_point.value,
                    next_point.value,
                    invlerp(this_point.time, next_point.time, current_t),
                    this_point.ease_type,
                );
                let point_view = self
                    .view_to_spline
                    .inverse()
                    .transform_pos(pos2(current_t, value));
                linepoints_view.push(point_view);
                current_screen_x += 1.0;
                current_t = self.view_to_spline.map_x(current_screen_x);
            }
            let point_view = self
                .view_to_spline
                .inverse()
                .transform_pos(pos2(next_point.time, next_point.value));
            circles_view.push(point_view);
            linepoints_view.push(point_view);
            if current_screen_x > self.view_area.max.x {
                break;
            }
            current_keypoint_idx += 1;
        }
        let line = PathShape::line(
            linepoints_view,
            Stroke::new(2.0, Color32::BLUE),
        );
        painter.add(line);
        for cir in circles_view {
            painter.circle_stroke(
                cir,
                2.0,
                Stroke::new(2.0, Color32::YELLOW),
            );
        }
        response
    }
    pub fn view_to_spline(&self) -> &RectTransform {
        &self.view_to_spline
    }
    pub fn visible_spline_area(&self) -> Rect {
        self.visible_spline_area
    }
    pub fn view_area(&self) -> Rect {
        self.view_area
    }
    pub fn spline_area(&self) -> Rect {
        self.spline_area
    }
}

pub trait SplineEditorAdapter {
    type Tween: Tween + Clone + std::fmt::Debug + PartialEq + Default + 'static;
    type Relevant: Clone + std::fmt::Debug + PartialEq + Default + 'static;
    type Path: ChartPath<Out = KeyPoint<Self::Tween, Self::Relevant>> + Copy;

    fn get_spline<'a>(&self, chart: &'a Chart) -> &'a Spline<Self::Tween, Self::Relevant>;
    fn path(&self, index: usize) -> Self::Path;

    fn edit_command(
        &self,
        path: Self::Path,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands;
    fn add_command(
        &self,
        index: usize,
        point: KeyPoint<Self::Tween, Self::Relevant>,
    ) -> ChartCommands;
    fn remove_command(&self, index: usize) -> ChartCommands;

    fn value_ui(&self, ui: &mut Ui, value: &mut Self::Tween) -> Response;
    fn relevant_ui(&self, ui: &mut Ui, _relevant: &mut Self::Relevant) -> Response {
        ui.label("N/A")
    }
}

pub struct SplineListEditor<A> {
    adapter: A,
}

impl<A: SplineEditorAdapter> SplineListEditor<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn show(&self, ui: &mut Ui, mut chart: Mut<Chart>, history: &mut EditHistory) {
        let spline = self.adapter.get_spline(&chart);
        let len = spline.len();
        let points = spline.points().clone();
        ui.group(|ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    for i in 0..=len {
                        ui.push_id(i, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button("+").clicked() {
                                    let new_point = if i > 0 {
                                        let mut p = points[i - 1].clone();
                                        if i < len {
                                            let next = &points[i];
                                            p.time = (p.time + next.time) / 2.0;
                                        } else {
                                            p.time += 1.0;
                                        }
                                        p
                                    } else if len > 0 {
                                        let mut p = points[0].clone();
                                        p.time -= 1.0;
                                        p
                                    } else {
                                        KeyPoint {
                                            time: 0.0,
                                            value: A::Tween::default(),
                                            ease_type: Default::default(),
                                            relevant: A::Relevant::default(),
                                        }
                                    };
                                    let _ = history.push(
                                        self.adapter.add_command(i, new_point),
                                        &mut *chart,
                                    );
                                }
                                ui.separator();
                            });

                            if i < len {
                                ui.horizontal(|ui| {
                                    ui.label(format!("Point {}", i));

                                    let path = self.adapter.path(i);

                                    ui.horizontal(|ui| {
                                        ui.label("Time:");
                                        edit_scope(
                                            ui,
                                            path,
                                            chart.reborrow(),
                                            history,
                                            "Edit Spline Point Time",
                                            |ui, point| {
                                                ui.add(
                                                    egui::DragValue::new(&mut point.time)
                                                        .speed(0.01),
                                                )
                                            },
                                            |path, point| self.adapter.edit_command(path, point),
                                        );
                                    });

                                    edit_scope(
                                        ui,
                                        path,
                                        chart.reborrow(),
                                        history,
                                        "Edit Spline Point Value",
                                        |ui, point| self.adapter.value_ui(ui, &mut point.value),
                                        |path, point| self.adapter.edit_command(path, point),
                                    );

                                    edit_scope(
                                        ui,
                                        path,
                                        chart.reborrow(),
                                        history,
                                        "Edit Spline Point Easing",
                                        |ui, point| {
                                            crate::widgets::enum_selector(&mut point.ease_type, ui)
                                        },
                                        |path, point| self.adapter.edit_command(path, point),
                                    );
                                    if !std::mem::size_of::<A::Relevant>() == 0 {
                                        ui.horizontal(|ui| {
                                            // ui.label("Relevant:");
                                            edit_scope(
                                                ui,
                                                path,
                                                chart.reborrow(),
                                                history,
                                                "Edit Spline Point Relevant",
                                                |ui, point| {
                                                    self.adapter
                                                        .relevant_ui(ui, &mut point.relevant)
                                                },
                                                |path, point| {
                                                    self.adapter.edit_command(path, point)
                                                },
                                            );
                                        });
                                    }
                                    if ui.button("🗑").clicked() {
                                        let _ = history.push(
                                            self.adapter.remove_command(i),
                                            &mut *chart,
                                        );
                                    }
                                });
                            }
                        });
                    }
                })
        });
    }
}
