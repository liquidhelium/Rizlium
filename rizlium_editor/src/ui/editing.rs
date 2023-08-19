use std::{ops::{Range, RangeInclusive}, fmt::format};

use bevy::prelude::info;
use egui::{epaint::PathShape, Color32, Pos2, Stroke, Ui};
use rizlium_chart::prelude::{Spline, Tween};

pub fn spline_editor_horizontal<R>(
    ui: &mut Ui,
    spline: &Spline<f32, R>,
    focus: &mut Option<usize>,
    cursor: f32,
    scale: &mut f32,
    vertical_view: &mut RangeInclusive<f32>,
) -> SplineEditorResponse {
    assert_ne!(*scale, 0.);
    egui::ScrollArea::horizontal()
        .drag_to_scroll(false)
        .id_source("spline_editor")
        .show_viewport(ui, |ui, view| {
            let min_time = view.left() / *scale;
            let max_time = view.right() / *scale;
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::ZERO;
                ui.horizontal_centered(|ui| {
                    let mut current_index = 0;
                    for window in spline.as_ref().windows(2) {
                        let this = &window[0];
                        let next = &window[1];
                        let width = (next.time - this.time) * *scale;
                        let (_, rect) = ui.allocate_space([width, ui.available_height()].into());
                        let remap =
                            |i: f32| egui::emath::remap(i, vertical_view.clone(), rect.y_range());
                        // if next.time > min_time || this.time < max_time {
                        let point_count = width.floor();
                        let last = Pos2::new(rect.right(), remap(next.value));
                        let first = Pos2::new(rect.left(), remap(this.value));
                        let iter = (0..point_count as usize)
                            .into_iter()
                            .map(|i| i as f32 / point_count)
                            .map(|x| {
                                Pos2::from([
                                    f32::lerp(rect.left(), rect.right(), x),
                                    f32::ease(
                                        remap(this.value),
                                        remap(next.value),
                                        x,
                                        this.ease_type,
                                    ),
                                ])
                            })
                            .chain(Some(last));
                        let shape =
                            PathShape::line(iter.collect(), Stroke::new(2., Color32::LIGHT_BLUE));
                        ui.painter().add(shape);
                        let rect = egui::Rect::from_center_size(first, egui::Vec2::splat(10.));
                        ui.painter()
                            .circle_stroke(first, 5., Stroke::new(2., Color32::DARK_BLUE));
                        let knob = ui
                            .interact(rect, ui.next_auto_id(), egui::Sense::click_and_drag())
                            .on_hover_cursor(egui::CursorIcon::Grab)
                            .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                        if knob.clicked() || knob.drag_started() {
                            *focus = Some(current_index);
                        } else if knob.drag_released() {
                            *focus = None;
                        }
                        current_index += 1;
                    }
                });
            });
            SplineEditorResponse {
                edit: None,
                focus_changed: false,
                value_range_changed: false,
                scale_changed: false,
                view_data: SplineViewData {
                    time_range: min_time..=max_time,
                },
                seek_to: None,
            }
        })
        .inner
}
#[derive(Clone)]
pub struct SplineEditorResponse {
    pub edit: Option<()>, // todo
    pub focus_changed: bool,
    pub value_range_changed: bool,
    pub scale_changed: bool,
    pub view_data: SplineViewData,
    pub seek_to: Option<f32>,
}

#[derive(Clone)]
pub struct SplineViewData {
    pub time_range: RangeInclusive<f32>,
}

pub fn timeline(
    ui: &mut Ui,
    cursor: f32,
    time_range: &mut RangeInclusive<f32>,
    scale: &mut f32,
) -> TimeLineResponse {
    const TIME_PIXEL_GAP: f32 = 50.;
    const MIN_TIME_GAP: f32 = 0.1;
    let gap_time = TIME_PIXEL_GAP / *scale;
    let gap_time_count_int = (gap_time / MIN_TIME_GAP).floor() as usize;
    let power = gap_time_count_int.next_power_of_two();
    let total_gaps = (time_range.end() / MIN_TIME_GAP).floor() as usize;
    egui::TopBottomPanel::top("timeline")
        .show_inside(ui, |ui| {
            let range_x = ui.available_rect_before_wrap().x_range();
            let range_y = ui.available_rect_before_wrap().y_range();
            let remap = |i: f32| egui::emath::remap(i, time_range.clone(), range_x.clone());
            // let rev_remap = |i| egui::emath::remap(i, ui.available_rect_before_wrap().x_range(), time_range.clone());

            for i in (0..=total_gaps+power).step_by(power) {
                let time = i as f32 * MIN_TIME_GAP;
                let x = remap(time);
                ui.allocate_ui_at_rect(
                    egui::Rect::from_x_y_ranges(
                        x..=x + 100.,
                        range_y.clone(),
                    ),
                    |ui| {
                        ui.label(format!("{:.2}", time));
                    },
                );
                ui.ctx().debug_painter().debug_rect(egui::Rect::from_x_y_ranges(
                    x..=x + 100.,
                    range_y.clone(),
                ), Color32::BLUE, "r");
            }
            TimeLineResponse {
                seek_to: None,
                range_changed: false,
                scale_changed: false,
            }
        })
        .inner
}

#[derive(Clone, Copy)]
pub struct TimeLineResponse {
    pub seek_to: Option<f32>,
    pub range_changed: bool,
    pub scale_changed: bool,
}
