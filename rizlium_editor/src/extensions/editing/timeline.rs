use std::ops::RangeInclusive;

use egui::{Align2, Color32, FontId, Id, Sense, Stroke, Ui};

const TIME_PIXEL_GAP: f32 = 50.;
const MIN_TIME_GAP: f32 = 1.;

pub fn timeline_horizontal(
    ui: &mut Ui,
    cursor: f32,
    time_range: &mut RangeInclusive<f32>,
    scale: &mut f32,
    view: egui::Rect,
    timeline_zone: egui::Rect,
) -> TimeLineResponse {
    let range_x = timeline_zone.x_range();
    let anoter_time = time_range.clone();
    let another_range = range_x.clone();
    let remap = |i| egui::remap(i, anoter_time.clone(), another_range.clone());
    let remap_reversed = |i| egui::remap(i, another_range.clone(), anoter_time.clone());
    cursor_v(ui, remap(cursor), view);
    for (time, x) in timeline_pos_iter(*scale, time_range.clone(), range_x.into()) {
        line_v(ui, remap(time), view, Stroke::new(1., Color32::DARK_GRAY));
        ui.painter().text(
            [x, timeline_zone.center().y].into(),
            Align2::LEFT_BOTTOM,
            time,
            FontId::default(),
            Color32::WHITE,
        );
    }
    let res = ui.interact(
        timeline_zone,
        Id::new("timeline_interact"),
        Sense::click_and_drag(),
    );

    TimeLineResponse {
        seek_to: ((res.is_pointer_button_down_on() && res.drag_delta().x != 0.) || res.clicked())
            .then(|| Some(remap_reversed(res.interact_pointer_pos().map(|p| p.x)?)))
            .flatten(),
        range_changed: false,
        scale_changed: false,
    }
}

pub fn timeline_vertical(
    ui: &mut Ui,
    cursor: f32,
    time_range: &mut RangeInclusive<f32>,
    scale: &mut f32,
    view: egui::Rect,
    timeline_zone: egui::Rect,
) -> TimeLineResponse {
    let range_x = timeline_zone.x_range();
    let range_y = timeline_zone.y_range();
    let another_time = time_range.clone();
    let another_range = range_y.clone();
    let remap = |i| egui::remap(i, another_time.clone(), another_range.clone());
    let remap_reversed = |i| egui::remap(i, another_range.clone(), another_time.clone());
    cursor_h(ui, remap(cursor), view);
    for (time, y) in timeline_pos_iter(*scale, time_range.clone(), range_y.into()) {
        line_h(ui, remap(time), view, Stroke::new(1., Color32::DARK_GRAY));
        ui.painter().text(
            [range_x.min, y].into(),
            Align2::LEFT_BOTTOM,
            time,
            FontId::default(),
            Color32::WHITE,
        );
    }
    let res = ui.interact(
        timeline_zone,
        Id::new("timeline_interact"),
        Sense::click_and_drag(),
    );

    TimeLineResponse {
        seek_to: ((res.is_pointer_button_down_on() && res.drag_delta().x != 0.) || res.clicked())
            .then(|| Some(remap_reversed(res.interact_pointer_pos().map(|p| p.x)?)))
            .flatten(),
        range_changed: false,
        scale_changed: false,
    }
}

const CURSOR_STROKE: Stroke = Stroke {
    width: 2.,
    color: Color32::GRAY,
};

fn cursor_v(ui: &mut Ui, x: f32, view: egui::Rect) {
    line_v(ui, x, view, CURSOR_STROKE);
}

fn line_v(ui: &mut Ui, x: f32, view: egui::Rect, stroke: Stroke) {
    ui.painter_at(view).vline(x, view.y_range(), stroke);
}

fn cursor_h(ui: &mut Ui, y: f32, view: egui::Rect) {
    line_h(ui, y, view, CURSOR_STROKE);
}

fn line_h(ui: &mut Ui, y: f32, view: egui::Rect, stroke: Stroke) {
    ui.painter_at(view).hline(view.x_range(), y, stroke);
}

fn timeline_pos_iter(
    scale: f32,
    time_range: RangeInclusive<f32>,
    pos_range: RangeInclusive<f32>,
) -> impl Iterator<Item = (f32, f32)> {
    let gap_time = TIME_PIXEL_GAP / scale;
    let gap_time_count = gap_time / MIN_TIME_GAP;
    let power = f32_next_power_of_two(gap_time_count);
    let gaps_end = time_range.end() / MIN_TIME_GAP;
    let gaps_start = ((time_range.start() / MIN_TIME_GAP).floor() / power).floor() * power;
    let remap = move |i: f32| egui::emath::remap(i, time_range.clone(), pos_range.clone());
    f32_range_step(gaps_start..=gaps_end, power)
        .map(move |i| (i * MIN_TIME_GAP, remap(i * MIN_TIME_GAP)))
}

///
/// ```rust
/// let near = (f32_next_power_of_two(0.2)-0.5).abs() <= 0.01;
/// assert!(near);
/// ```
fn f32_next_power_of_two(val: f32) -> f32 {
    2.0f32.powf(val.log2().ceil())
}

fn f32_range_step(range: RangeInclusive<f32>, step: f32) -> impl Iterator<Item = f32> {
    assert!(step > 0.);
    let mut curr = *range.start();
    let sign = (range.end() - range.start()).signum();
    let former = range.start().min(*range.end());
    let latter = range.start().max(*range.end());
    std::iter::from_fn(move || {
        let ret = if (former..=latter).contains(&curr) {
            Some(curr)
        } else {
            None
        };
        curr += sign * step;
        ret
    })
}

#[derive(Clone, Copy)]
pub struct TimeLineResponse {
    pub seek_to: Option<f32>,
    pub range_changed: bool,
    pub scale_changed: bool,
}
