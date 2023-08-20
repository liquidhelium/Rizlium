use std::ops::RangeInclusive;

use bevy::prelude::info;
use egui::{epaint::PathShape, Color32, Pos2, Sense, Stroke, Ui};
use rizlium_chart::prelude::{Spline, Tween};

pub fn spline_editor_horizontal<R>(
    ui: &mut Ui,
    spline: &Spline<f32, R>,
    focus: &mut Option<usize>,
    _cursor: f32,
    scale: &mut [f32; 2],
    scroll_to_first: bool,
) -> SplineEditorResponse {
    assert_ne!(*scale, [0., 0.]);
    let x_range = ui.available_rect_before_wrap().x_range();
    let y_range = ui.available_rect_before_wrap().y_range();
    egui::ScrollArea::new([true, true])
        .drag_to_scroll(false)
        .id_source("spline_editor")
        .show_viewport(ui, |ui, view| {
            let scale_x = scale[0];
            let scale_y = scale[1];
            let min_time = view.left() / scale_x;
            let max_time = view.right() / scale_x;
            let min_value = view.bottom() / scale_y;
            let max_value = view.top() / scale_y;
            let remap_x = |i: f32| egui::emath::remap(i, min_time..=max_time, x_range.clone());
            let remap_y = |i: f32| egui::emath::remap(i, min_value..=max_value, y_range.clone());
            let remap_point = |pos: Pos2| Pos2::new(remap_x(pos.x), remap_y(pos.y));
            if let Some(point) = spline.last() {
                ui.allocate_rect(
                    egui::Rect::from_two_pos(
                        remap_point([0., 0.].into()),
                        remap_point(point.as_slice().into()),
                    ),
                    Sense::hover(),
                );
            }
            ui.scope(|ui| {
                ui.style_mut().spacing.item_spacing = egui::Vec2::ZERO;
                ui.horizontal_centered(|ui| {
                    for (current_index, window) in spline.as_ref().windows(2).enumerate() {
                        let this = &window[0];
                        let next = &window[1];
                        let width = (next.time - this.time) * scale_x;
                        let this_pos = remap_point(this.as_slice().into());
                        let next_pos = remap_point(next.as_slice().into());
                        let rect = egui::Rect::from_two_pos(this_pos, next_pos);
                        ui.allocate_rect(rect, egui::Sense::hover());
                        if next.time > min_time || this.time < max_time {
                            let point_count = width.floor();
                            let iter = (0..point_count as usize)
                                .map(|i| i as f32 / point_count)
                                .map(|x| {
                                    Pos2::from([
                                        f32::lerp(rect.left(), rect.right(), x),
                                        f32::ease(this_pos.y, next_pos.y, x, this.ease_type),
                                    ])
                                })
                                .chain(Some(next_pos));
                            let shape = PathShape::line(
                                iter.collect(),
                                Stroke::new(2., Color32::LIGHT_BLUE),
                            );
                            ui.painter().add(shape);
                            let rect =
                                egui::Rect::from_center_size(this_pos, egui::Vec2::splat(10.));
                            ui.painter().circle_stroke(
                                this_pos,
                                5.,
                                Stroke::new(2., Color32::DARK_BLUE),
                            );
                            let knob = ui
                                .interact(rect, ui.next_auto_id(), egui::Sense::click_and_drag())
                                .on_hover_cursor(egui::CursorIcon::Grab)
                                .on_hover_and_drag_cursor(egui::CursorIcon::Grabbing);
                            if knob.clicked() || knob.drag_started() {
                                *focus = Some(current_index);
                            } else if knob.drag_released() {
                                *focus = None;
                            }
                            if current_index == 0 && scroll_to_first {
                                ui.scroll_to_rect(rect, Some(egui::Align::Center));
                            }
                        }
                    }
                });
            });
            let delta_scale_x = ui.input(|i| i.zoom_delta());
            // let delta_scale_y = ui.input(|i| {
            //     i.multi_touch().map_or({
            //         i.modifiers.alt.then_some(i.scroll_delta.y).map(f)
            //     }, f)
            // });
            let scale_changed = if delta_scale_x == 1. {
                false
            } else {
                info!("scaling: {delta_scale_x}");
                scale[0] *= delta_scale_x;
                true
            };
            SplineEditorResponse {
                edit: None,
                focus_changed: false,
                value_range_changed: false,
                scale_changed,
                view_rect: egui::Rect::from_x_y_ranges(min_time..=max_time, min_value..=max_value),
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
    pub view_rect: egui::Rect,
    pub seek_to: Option<f32>,
}

pub fn timeline_horizontal(
    ui: &mut Ui,
    cursor: f32,
    time_range: &mut RangeInclusive<f32>,
    scale: &mut f32,
    view: egui::Rect,
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
            cursor_v(ui, remap(cursor), view);

            for i in (0..=total_gaps + power).step_by(power) {
                let time = i as f32 * MIN_TIME_GAP;
                let x = remap(time);
                ui.allocate_ui_at_rect(
                    egui::Rect::from_x_y_ranges(x..=x + 100., range_y.clone()),
                    |ui| {
                        ui.label(format!("{time:.2}"));
                    },
                );
            }
            TimeLineResponse {
                seek_to: None,
                range_changed: false,
                scale_changed: false,
            }
        })
        .inner
}

fn cursor_v(ui: &mut Ui, x: f32, view: egui::Rect) {
    ui.painter_at(view)
        .vline(x, view.y_range(), Stroke::new(2., Color32::GRAY));
}

#[derive(Clone, Copy)]
pub struct TimeLineResponse {
    pub seek_to: Option<f32>,
    pub range_changed: bool,
    pub scale_changed: bool,
}
