use egui::{
    emath::RectTransform, epaint::PathShape, pos2, remap, Color32, Pos2, Rect, Response, Sense,
    Stroke, Ui,
};
use rizlium_chart::{
    chart::invlerp,
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
        let screen_area = ui.ctx().screen_rect();
        let view_area = ui.available_rect_before_wrap();
        let view_area = view_area.translate(-view_area.left_bottom().to_vec2());
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
        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
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
