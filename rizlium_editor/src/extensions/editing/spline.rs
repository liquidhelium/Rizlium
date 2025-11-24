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
    screen2view: RectTransform,
    view2visible: RectTransform,
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
            screen2view: RectTransform::from_to(screen_area, view_area),
            view2visible: RectTransform::from_to(view_area, visible_spline_area),
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
        let mut current_segment_index = 0;
        let mut current_t = self.view2visible.map_x(current_segment_index as f32);
        let mut current_keypoint_idx = match self.spline.keypoint_at(current_t) {
            Ok(idx) => {
                let point = self.spline.points().get(idx).unwrap();
                let point_view = self
                    .view2visible
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
                        .view2visible
                        .inverse()
                        .transform_pos(pos2(point.time, point.value));
                    circles_view.push(point_view);
                    linepoints_view.push(point_view);
                    current_t = point.time + 0.01;
                    current_segment_index =
                        self.view2visible.inverse().map_x(current_t).ceil() as usize + 2;
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
                    .view2visible
                    .inverse()
                    .transform_pos(pos2(current_t, value));
                linepoints_view.push(point_view);
                current_segment_index += 1;
                current_t = self.view2visible.map_x(current_segment_index as f32);
            }
            let point_view = self
                .view2visible
                .inverse()
                .transform_pos(pos2(next_point.time, next_point.value));
            circles_view.push(point_view);
            linepoints_view.push(point_view);
            if current_segment_index > self.view_area.width().ceil() as usize {
                break;
            }
            current_keypoint_idx += 1;
        }
        let line = PathShape::line(
            linepoints_view
                .into_iter()
                .map(|p| self.screen2view.inverse().transform_pos(p))
                .collect(),
            Stroke::new(2.0, Color32::BLUE),
        );
        painter.add(line);
        for cir in circles_view {
            painter.circle_stroke(
                self.screen2view.inverse().transform_pos(cir),
                2.0,
                Stroke::new(2.0, Color32::YELLOW),
            );
        }
        response
    }
    pub fn screen2view(&self) -> &RectTransform {
        &self.screen2view
    }
    pub fn view2visible(&self) -> &RectTransform {
        &self.view2visible
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

pub struct SplineListEditor<P, T: Tween, R, F, C, V, Rel, G> {
    get_spline: G,
    path_builder: F,
    command_builder: C,
    value_ui: V,
    relevant_ui: Rel,
    _phantom: std::marker::PhantomData<(P, T, R)>,
}

impl<P, T: Tween, R, F, C, V, G>
    SplineListEditor<P, T, R, F, C, V, fn(&mut Ui, &mut R) -> Response, G>
where
    P: ChartPath<Out = KeyPoint<T, R>> + Copy,
    T: Tween + Clone + std::fmt::Debug + PartialEq + 'static,
    R: Clone + std::fmt::Debug + PartialEq + 'static,
    F: Fn(usize) -> P,
    C: Fn(P, KeyPoint<T, R>) -> ChartCommands + Clone,
    V: Fn(&mut Ui, &mut T) -> Response,
    G: Fn(&Chart) -> &Spline<T, R>,
{
    pub fn new(get_spline: G, path_builder: F, command_builder: C, value_ui: V) -> Self {
        Self {
            get_spline,
            path_builder,
            command_builder,
            value_ui,
            relevant_ui: |ui, _| ui.label("N/A"),
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<P, T: Tween, R, F, C, V, Rel, G> SplineListEditor<P, T, R, F, C, V, Rel, G>
where
    P: ChartPath<Out = KeyPoint<T, R>> + Copy,
    T: Tween + Clone + std::fmt::Debug + PartialEq + 'static,
    R: Clone + std::fmt::Debug + PartialEq + 'static,
    F: Fn(usize) -> P,
    C: Fn(P, KeyPoint<T, R>) -> ChartCommands + Clone,
    V: Fn(&mut Ui, &mut T) -> Response,
    Rel: Fn(&mut Ui, &mut R) -> Response,
    G: Fn(&Chart) -> &Spline<T, R>,
{
    pub fn new_relevant(
        get_spline: G,
        path_builder: F,
        command_builder: C,
        value_ui: V,
        relevant_ui: Rel,
    ) -> Self {
        Self {
            get_spline,
            path_builder,
            command_builder,
            value_ui,
            relevant_ui,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn show(&self, ui: &mut Ui, mut chart: Mut<Chart>, history: &mut EditHistory) {
        let len = (self.get_spline)(&chart).len();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for i in 0..len {
                ui.push_id(i, |ui| {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("Point {}", i));
                        });

                        let path = (self.path_builder)(i);

                        ui.horizontal(|ui| {
                            ui.label("Time:");
                            edit_scope(
                                ui,
                                path,
                                chart.reborrow(),
                                history,
                                |ui, point| {
                                    ui.add(egui::DragValue::new(&mut point.time).speed(0.01))
                                },
                                self.command_builder.clone(),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Value:");
                            edit_scope(
                                ui,
                                path,
                                chart.reborrow(),
                                history,
                                |ui, point| (self.value_ui)(ui, &mut point.value),
                                self.command_builder.clone(),
                            );
                        });

                        ui.horizontal(|ui| {
                            ui.label("Easing:");
                            edit_scope(
                                ui,
                                path,
                                chart.reborrow(),
                                history,
                                |ui, point| crate::widgets::enum_selector(&mut point.ease_type, ui),
                                self.command_builder.clone(),
                            );
                        });
                        if !std::mem::size_of::<R>() == 0 {
                            ui.horizontal(|ui| {
                                // ui.label("Relevant:");
                                edit_scope(
                                    ui,
                                    path,
                                    chart.reborrow(),
                                    history,
                                    |ui, point| (self.relevant_ui)(ui, &mut point.relevant),
                                    self.command_builder.clone(),
                                );
                            });
                        }
                    });
                });
            }
        });
    }
}
