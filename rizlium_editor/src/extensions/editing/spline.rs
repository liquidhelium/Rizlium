use bevy::{
    log::info,
    math::{vec2, Vec2}, utils::default,
};
use egui::{
    emath::RectTransform, epaint::PathShape, remap, Color32, Layout, NumExt, Pos2, Rangef, Rect, Sense, Stroke, Ui
};
use rizlium_chart::prelude::{Spline, Tween};

use super::timeline::timeline_horizontal;

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
            let remap_x = |i: f32| egui::emath::remap(i, min_time..=max_time, x_range);
            let remap_y = |i: f32| egui::emath::remap(i, min_value..=max_value, y_range);
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

pub fn spline_vertical<R>(
    ui: &mut Ui,
    spline: &Spline<f32, R>,
    scale: &mut Vec2,
    visible_time_range: &mut Rangef,
    visible_x_range: &mut Rangef,
) {
    let content_clip_rect = ui.available_rect_before_wrap();
    let end_time = spline.end_time().unwrap_or_default() + 20.;
    let y_size = (end_time * scale.y).at_least(ui.available_height());
    scale.y = scale.y.at_least(ui.available_height() / end_time);
    let local_full_pixel_range_y = Rangef::new(0., y_size);
    let abs_visible_pixel_range_y = content_clip_rect.y_range();
    let time_range = Rangef::new(0., end_time);
    // clamp y_range limit
    clamp_into(&time_range, visible_time_range);
    // 接下来要由visible 和end_time的相对位置找出child_ui的大小.
    let abs_full_pixel_range_y = remap_range(time_range, visible_time_range, &abs_visible_pixel_range_y);
    let local_visible_pixel_range_y = Rangef::new(visible_time_range.min *scale.y, visible_time_range.max * scale.y);
    let mut child_ui = ui.child_ui(
        Rect::from_x_y_ranges(content_clip_rect.x_range(), abs_full_pixel_range_y),
        *ui.layout(),
    );
    child_ui.set_clip_rect(content_clip_rect);
    if child_ui
        .interact(content_clip_rect, "spline_vertical".into(), Sense::hover())
        .hovered()
    {
        if let Some(delta) = child_ui.ctx().input(|input| {
            (!input.smooth_scroll_delta.y.eq(&0.)).then_some(input.smooth_scroll_delta.y)
        }) {
            *visible_time_range = range_plus(*visible_time_range, delta / scale.y);
        }
    }
    
}

fn clamp_into(source: &Rangef, range: &mut Rangef) {
    match (source.min <= range.min, range.max <= source.max) {
        (false, true) => {
            let delta = source.min - range.min; // 一个正数
            range.min = source.min;
            range.max = (range.max + delta).at_most(source.max);
        }
        (true, false) => {
            let delta = source.max - range.max; // 一个负数
            range.max = source.max;
            range.min = (range.min + delta).at_least(source.min);
        }
        (false, false) => *range = *source,
        _ => (),
    }
}

fn remap_range(x: Rangef, from: &Rangef, to: &Rangef) -> Rangef {
    Rangef::new(remap(x.min, from, to), remap(x.max, from, to))
}

fn range_plus(range: Rangef, x: f32) -> Rangef {
    Rangef {
        min: range.min + x,
        max: range.max + x,
    }
}

struct Viewport {
    inner_to_show: RectTransform,
    /// Mainly used for mouse.
    show_to_inner: RectTransform,
}

struct RelativeArea {
    inner_rect: Option<Rect>
}

impl RelativeArea {
    fn show_inside(self, ui:&mut Ui, show_ui: impl FnOnce(&mut Ui, &Viewport)) {
        let show_rect = ui.available_rect_before_wrap();
        let inner_rect = self.inner_rect.unwrap_or(show_rect);
        let inner_show_transform = RectTransform::from_to(inner_rect, show_rect);
        let show_inner_transform = inner_show_transform.inverse();
        
    }
}