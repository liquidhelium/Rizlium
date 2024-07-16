use bevy::{
    log::{debug, info},
    math::{vec2, Vec2},
    utils::default,
};
use egui::{
    emath::RectTransform, epaint::PathShape, pos2, remap, Color32, Layout, NumExt, Pos2, Rangef, Rect, Response, Sense, Stroke, Ui
};
use rizlium_chart::{
    chart::invlerp,
    prelude::{Spline, Tween},
};

use super::timeline::timeline_horizontal;

#[derive(Clone)]
pub struct SplineEditorResponse {
    pub edit: Option<()>, // todo
    pub to_focus: Option<usize>,
    pub value_range_changed: bool,
    pub scale_changed: bool,
    pub view_rect: egui::Rect,
    pub seek_to: Option<f32>,
}

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

pub struct SplineView<'a, R> {
    spline: &'a Spline<f32, R>,
    screen_area: Rect,
    view_area: Rect,
    spline_area: Rect,
    visible_spline_area: Rect,
    screen2view: RectTransform,
    view2visible: RectTransform,
}

impl<'a, R> SplineView<'a, R> {
    pub fn new(ui: &mut Ui, spline: &'a Spline<f32, R>, visible_spline_area: Option<Rect>) -> Self {
        let screen_area = ui.ctx().screen_rect();
        let view_area = ui.available_rect_before_wrap();
        let view_area = view_area.translate(-view_area.left_bottom().to_vec2());
        let spline_area = {
            let (time_start, value_min) = spline.first().map_or((0., 0.), |f| (f.time, f.value));
            let (time_end, value_max) = spline
                .last()
                .map_or((0., 0.), |l| (l.time + 0.0, l.value + 0.0));
            Rect::from_two_pos(pos2(time_start, value_min), pos2(time_end, value_max))
        };
        let visible_spline_area = visible_spline_area.unwrap_or(spline_area.expand2(egui::Vec2 {
            x: spline_area.width() / 2.,
            y: 100.,
        }));
        Self {
            spline,
            screen_area,
            view_area,
            spline_area,
            visible_spline_area,
            screen2view: RectTransform::from_to(screen_area, view_area),
            view2visible: RectTransform::from_to(view_area, visible_spline_area),
        }
    }

    pub fn ui(&self, ui: &mut Ui) -> Option<Response>{
        if self.spline.is_empty() {
            return None;
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
                    return None;
                }
            }
        };
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
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
                    this_point.ease_type
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
        Some(response)
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
}
