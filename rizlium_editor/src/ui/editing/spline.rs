use bevy::prelude::info;
use egui::{epaint::PathShape, Color32, Pos2, Rect, Stroke, Ui};
use rizlium_chart::prelude::{Spline, Tween};

use crate::ui::editing::timeline_horizontal;

pub fn spline_editor_horizontal<R>(
    ui: &mut Ui,
    spline: &Spline<f32, R>,
    _focus: Option<usize>,
    cursor: f32,
    scale: &mut [f32; 2],
    scroll_to_first: bool,
) -> SplineEditorResponse {
    assert_ne!(*scale, [0., 0.]);
    let x_range = ui.available_rect_before_wrap().x_range();
    let y_range = ui.available_rect_before_wrap().y_range();
    let editor_area = ui.available_rect_before_wrap();
    egui::ScrollArea::new([true; 2])
        .drag_to_scroll(false)
        .id_source("spline_editor_h")
        .auto_shrink([false; 2])
        .show_viewport(ui, |ui, view| {
            let mut scale_x = scale[0];
            let scale_y = scale[1];
            let min_time = view.left() / scale_x;
            let max_time = view.right() / scale_x;
            let min_value = view.bottom() / scale_y;
            let max_value = view.top() / scale_y;
            let remap_x = |i: f32| egui::emath::remap(i, min_time..=max_time, x_range.clone());
            let remap_y = |i: f32| egui::emath::remap(i, min_value..=max_value, y_range.clone());
            let remap_point = |pos: Pos2| Pos2::new(remap_x(pos.x), remap_y(pos.y));
            if let Some(point) = spline.last() {
                ui.set_width(point.time * scale_x);
                ui.set_height(point.value * scale_y);
            }
            let mut to_focus = None;
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
                            to_focus =
                                (knob.clicked() || knob.drag_started()).then_some(current_index);
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
            let response = timeline_horizontal(
                ui,
                cursor,
                &mut (min_time..=max_time),
                &mut scale_x,
                editor_area,
                Rect::from_two_pos(
                    editor_area.left_top(),
                    [editor_area.right(), editor_area.top() + 20.].into(),
                ),
            );
            SplineEditorResponse {
                edit: None,
                to_focus,
                value_range_changed: false,
                scale_changed,
                view_rect: egui::Rect::from_x_y_ranges(min_time..=max_time, min_value..=max_value),
                seek_to: response.seek_to,
            }
        })
        .inner
}
#[derive(Clone)]
pub struct SplineEditorResponse {
    pub edit: Option<()>, // todo
    pub to_focus: Option<usize>,
    pub value_range_changed: bool,
    pub scale_changed: bool,
    pub view_rect: egui::Rect,
    pub seek_to: Option<f32>,
}
