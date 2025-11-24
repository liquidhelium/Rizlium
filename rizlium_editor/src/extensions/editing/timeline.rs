use std::ops::RangeInclusive;

use bevy::prelude::*;
use egui::{Align2, Color32, FontId, Id, Sense, Stroke, Ui};
use helium_framework::prelude::*;
use rizlium_render::ChartProvider as _;
use rust_i18n::t;

use crate::extensions::inspector::{ChartItem, SelectedItem};
use crate::project::ProjectState;

use super::spline::{Orientation, SplineView};

pub struct TimelinePlugin;

impl Plugin for TimelinePlugin {
    fn build(&self, app: &mut App) {
        app.register_tab(
            "edit.timeline",
            t!("edit.timeline.tab"),
            timeline_tab,
            ProjectState::has_chart_system(),
        );
    }
}

#[derive(Default)]
struct TimelineState {
    time_range: Option<RangeInclusive<f32>>,
    vertical_scroll: f32,
    follow_cursor: bool,
    lock_ratio: Option<f32>,
}

fn timeline_tab(
    InMut(ui): InMut<Ui>,
    chart_state: Res<ProjectState>,
    selected_item: Res<SelectedItem>,
    mut state: Local<TimelineState>,
    game_time: Res<rizlium_render::GameTime>,
    mut time_control: EventWriter<crate::time_and_audio::TimeControlEvent>,
    chart_cache: Res<rizlium_render::GameChartCache>,
) {
    let chart = chart_state.chart();
    let mut tracks: Vec<(&str, &rizlium_chart::prelude::Spline<f32>)> = vec![];

    if let Some(item) = &selected_item.item {
        match item {
            ChartItem::BpmControl => {
                tracks.push(("BPM", &chart.bpm));
            }
            ChartItem::CameraControl => {
                tracks.push(("Cam Scale", &chart.cam_scale));
                tracks.push(("Cam Move", &chart.cam_move));
            }
            _ => {}
        }
    }

    if tracks.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label(t!("edit.timeline.select_hint"));
        });
        return;
    }

    let mut time_range = state.time_range.clone().unwrap_or(0.0..=10.0);
    let mut vertical_scroll = state.vertical_scroll;
    let cursor_time = **game_time;

    let total_duration = 300.0; // TODO: Calculate from chart
    let track_height = 100.0;
    let content_height = tracks.len() as f32 * track_height;

    let old_range = time_range.clone();
    let old_follow = state.follow_cursor;

    let (interacted, seek_to) = fl_timeline::fl_timeline_ui(
        ui,
        &mut time_range,
        total_duration,
        &mut vertical_scroll,
        content_height,
        &Default::default(),
        cursor_time,
        &mut state.follow_cursor,
        |ctx| {
            // Draw separator line on the right side of the left panel
            ctx.ui.painter().line_segment(
                [ctx.rect.right_top(), ctx.rect.right_bottom()],
                ctx.ui.style().visuals.window_stroke,
            );

            for (i, (name, _)) in tracks.iter().enumerate() {
                let y = i as f32 * track_height;
                let screen_y = ctx.y_to_screen(y);
                // Simple culling
                if screen_y + track_height < ctx.rect.top() || screen_y > ctx.rect.bottom() {
                    continue;
                }

                let rect = egui::Rect::from_min_size(
                    egui::pos2(ctx.rect.left(), screen_y),
                    egui::vec2(ctx.rect.width(), track_height),
                );

                ctx.ui
                    .allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(*name);
                        });
                    });

                ctx.ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_bottom()],
                    egui::Stroke::new(1.0, ctx.ui.visuals().extreme_bg_color),
                );
            }
        },
        |ctx| {
            for (i, (_, spline)) in tracks.iter().enumerate() {
                let y = i as f32 * track_height;
                let screen_y = ctx.y_to_screen(y);
                // Simple culling
                if screen_y + track_height < ctx.rect.top() || screen_y > ctx.rect.bottom() {
                    continue;
                }

                let rect = egui::Rect::from_min_size(
                    egui::pos2(ctx.rect.left(), screen_y),
                    egui::vec2(ctx.rect.width(), track_height),
                );

                ctx.ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_bottom()],
                    egui::Stroke::new(1.0, ctx.ui.visuals().extreme_bg_color),
                );

                ctx.ui
                    .allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                        ui.set_clip_rect(rect);

                        // Auto-fit value range
                        let (min_val, max_val) = spline
                            .points()
                            .iter()
                            .fold((f32::INFINITY, f32::NEG_INFINITY), |(min, max), p| {
                                (min.min(p.value), max.max(p.value))
                            });
                        let (min_val, max_val) = if min_val.is_infinite() {
                            (0.0, 1.0)
                        } else {
                            (min_val, max_val)
                        };

                        let range = max_val - min_val;
                        let margin = if range == 0.0 { 1.0 } else { range * 0.1 };

                        let visible_area = egui::Rect::from_min_max(
                            egui::pos2(*ctx.visible_time.start(), min_val - margin),
                            egui::pos2(*ctx.visible_time.end(), max_val + margin),
                        );

                        let sv = SplineView::new(
                            ui,
                            spline,
                            Some(visible_area),
                            Orientation::Horizontal,
                        );
                        sv.ui(ui);
                    });
            }
        },
    );

    // Logic for follow cursor
    let view_width = time_range.end() - time_range.start();

    let mut cursor_time = game_time.0;
    if let Some(seek_time) = seek_to {
        let real_time = chart_cache.remap_beat(seek_time);
        time_control.write(crate::time_and_audio::TimeControlEvent::Seek(real_time));
        cursor_time = seek_time;
    }

    let current_ratio = (cursor_time - time_range.start()) / view_width;

    if interacted {
        // User manually scrolled/zoomed or seeked
        state.lock_ratio = Some(current_ratio);
    } else if state.follow_cursor {
        if !old_follow {
            state.lock_ratio = Some(current_ratio);
        }
        let lock_ratio = state.lock_ratio.unwrap_or(current_ratio);

        let target_start = cursor_time - lock_ratio * view_width;
        let new_start = target_start.clamp(0.0, total_duration - view_width);
        let new_end = new_start + view_width;
        time_range = new_start..=new_end;
    }

    state.time_range = Some(time_range);
    state.vertical_scroll = vertical_scroll;
}

const TIME_PIXEL_GAP: f32 = 100.;
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
    let another_range = range_x;
    let remap = |i| egui::remap(i, anoter_time.clone(), another_range);
    let remap_reversed = |i| egui::remap(i, another_range, anoter_time.clone());
    cursor_v(ui, remap(cursor), view);

    let gap_time = TIME_PIXEL_GAP / *scale;
    let gap_time_count = gap_time / MIN_TIME_GAP;
    let power = f32_next_power_of_two(gap_time_count);
    let sub_step = power / 4.0;

    let start_k = (*time_range.start() / MIN_TIME_GAP / sub_step).floor() as i64;
    let end_k = (*time_range.end() / MIN_TIME_GAP / sub_step).ceil() as i64;

    for k in start_k..=end_k {
        let time = k as f32 * sub_step * MIN_TIME_GAP;
        let x = remap(time);

        let spacing = if k % 4 == 0 {
            power * MIN_TIME_GAP * *scale
        } else if k % 2 == 0 {
            (power / 2.0) * MIN_TIME_GAP * *scale
        } else {
            (power / 4.0) * MIN_TIME_GAP * *scale
        };

        let fade_start = 100.0;
        let fade_end = 20.0;
        let alpha = ((spacing - fade_end) / (fade_start - fade_end)).clamp(0.0, 1.0);

        if alpha <= 0.0 {
            continue;
        }

        let line_color = Color32::DARK_GRAY.linear_multiply(alpha);
        let text_color = Color32::WHITE.linear_multiply(alpha);
        let font_size = 10.0 + 4.0 * alpha;

        line_v(ui, x, view, Stroke::new(1., line_color));
        ui.painter().text(
            [x, timeline_zone.min.y].into(),
            Align2::CENTER_TOP,
            time.to_string(),
            FontId::proportional(font_size),
            text_color,
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
    let another_range = range_y;
    let remap = |i| egui::remap(i, another_time.clone(), another_range);
    let remap_reversed = |i| egui::remap(i, another_range, another_time.clone());
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

pub mod fl_timeline {
    use super::*;
    use egui::{pos2, vec2, Color32, Id, Rect, Sense, Stroke, StrokeKind, Ui};
    use std::ops::RangeInclusive;

    pub struct FlTimelineConfig {
        pub header_width: f32,
        pub timeline_height: f32,
        pub scroll_bar_height: f32,
    }

    impl Default for FlTimelineConfig {
        fn default() -> Self {
            Self {
                header_width: 150.0,
                timeline_height: 30.0,
                scroll_bar_height: 20.0,
            }
        }
    }

    pub struct FlLeftPanelContext<'a> {
        pub ui: &'a mut Ui,
        pub rect: Rect,
        pub visible_y: RangeInclusive<f32>,
    }

    impl<'a> FlLeftPanelContext<'a> {
        pub fn y_to_screen(&self, y: f32) -> f32 {
            self.rect.top() + (y - self.visible_y.start())
        }
    }

    pub struct FlContentContext<'a> {
        pub ui: &'a mut Ui,
        pub rect: Rect,
        pub visible_time: RangeInclusive<f32>,
        pub visible_y: RangeInclusive<f32>,
    }

    impl<'a> FlContentContext<'a> {
        pub fn time_to_screen(&self, time: f32) -> f32 {
            egui::remap(time, self.visible_time.clone(), self.rect.x_range())
        }
        pub fn y_to_screen(&self, y: f32) -> f32 {
            self.rect.top() + (y - self.visible_y.start())
        }
        pub fn point_to_screen(&self, time: f32, y: f32) -> egui::Pos2 {
            egui::pos2(self.time_to_screen(time), self.y_to_screen(y))
        }
    }

    pub fn fl_timeline_ui(
        ui: &mut Ui,
        time_range: &mut RangeInclusive<f32>,
        total_duration: f32,
        vertical_scroll: &mut f32,
        content_height: f32,
        config: &FlTimelineConfig,
        cursor_time: f32,
        follow_cursor: &mut bool,
        mut draw_left_panel: impl FnMut(FlLeftPanelContext),
        mut draw_content: impl FnMut(FlContentContext),
    ) -> (bool, Option<f32>) {
        let mut available_size = ui.available_size();
        let clip_rect = ui.clip_rect();
        let mut interacted = false;
        let mut seek_to = None;

        // If inside a ScrollArea, available_size might be infinite or very large.
        // We need to constrain it to the visible area (clip_rect) because fl_timeline_ui
        // implements its own virtual scrolling and assumes content_rect represents the visible viewport.

        // Constrain width
        if available_size.x.is_infinite() || available_size.x > clip_rect.width() {
            available_size.x = clip_rect.width();
        }

        // Constrain height
        if available_size.y.is_infinite() || available_size.y > clip_rect.height() {
            available_size.y = (clip_rect.bottom() - ui.cursor().min.y).max(100.0);
        }

        let top_bar_rect = Rect::from_min_size(
            ui.cursor().min,
            vec2(available_size.x, config.scroll_bar_height),
        );

        // 1. Top Scroll/Zoom Bar
        let scroll_bar_rect = Rect::from_min_size(
            top_bar_rect.min + vec2(config.header_width, 0.0),
            vec2(
                top_bar_rect.width() - config.header_width,
                top_bar_rect.height(),
            ),
        );
        if zoom_scroll_bar(ui, scroll_bar_rect, time_range, total_duration) {
            interacted = true;
        }

        let main_area_rect = Rect::from_min_size(
            top_bar_rect.left_bottom(),
            available_size - vec2(0.0, config.scroll_bar_height),
        );

        // Grid Layout
        let header_width = config.header_width;
        let timeline_height = config.timeline_height;

        let corner_rect = Rect::from_min_size(
            top_bar_rect.min,
            vec2(header_width, timeline_height + config.scroll_bar_height),
        );

        ui.allocate_new_ui(egui::UiBuilder::new().max_rect(corner_rect), |ui| {
            ui.centered_and_justified(|ui| {
                if ui.selectable_label(*follow_cursor, "S").clicked() {
                    *follow_cursor = !*follow_cursor;
                }
            });
        });

        let timeline_rect = Rect::from_min_size(
            main_area_rect.min + vec2(header_width, 0.0),
            vec2(main_area_rect.width() - header_width, timeline_height),
        );
        let left_panel_rect = Rect::from_min_size(
            main_area_rect.min + vec2(0.0, timeline_height),
            vec2(header_width, main_area_rect.height() - timeline_height),
        );
        let content_rect = Rect::from_min_size(
            main_area_rect.min + vec2(header_width, timeline_height),
            vec2(
                main_area_rect.width() - header_width,
                main_area_rect.height() - timeline_height,
            ),
        );

        // Draw Timeline
        {
            // Draw timeline background (top and bottom borders and fill only)
            ui.painter().line_segment(
                [timeline_rect.left_top(), timeline_rect.right_top()],
                egui::Stroke::new(1.0, ui.visuals().extreme_bg_color),
            );
            ui.painter().line_segment(
                [timeline_rect.left_bottom(), timeline_rect.right_bottom()],
                egui::Stroke::new(1.0, ui.visuals().extreme_bg_color),
            );
            ui.painter()
                .rect_filled(timeline_rect, 0.0, ui.visuals().extreme_bg_color);

            let response = ui
                .allocate_new_ui(
                    egui::UiBuilder::new()
                        .max_rect(timeline_rect)
                        .layout(egui::Layout::left_to_right(egui::Align::Min)),
                    |ui| {
                        ui.set_clip_rect(timeline_rect.union(content_rect));

                        let duration = time_range.end() - time_range.start();
                        let pixels_per_unit = timeline_rect.width() / duration;
                        let mut scale = pixels_per_unit;

                        super::timeline_horizontal(
                            ui,
                            cursor_time,
                            time_range,
                            &mut scale,
                            timeline_rect.union(content_rect),
                            timeline_rect,
                        )
                    },
                )
                .inner;

            if let Some(t) = response.seek_to {
                seek_to = Some(t);
                interacted = true;
            }
        }

        // Handle Content Input (Scrolling)
        let content_response = ui.interact(
            content_rect,
            Id::new("fl_content_area"),
            Sense::click_and_drag(),
        );
        if content_response.dragged() {
            let delta = content_response.drag_delta();

            // Horizontal Scroll (Time)
            let duration = time_range.end() - time_range.start();
            let dt = -delta.x / content_rect.width() * duration;
            let new_start = (*time_range.start() + dt).max(0.0);
            let new_end = (new_start + duration).min(total_duration);
            // Re-clamp start if end hit max
            let new_start = (new_end - duration).max(0.0);
            *time_range = new_start..=new_end;
            interacted = true;

            // Vertical Scroll
            *vertical_scroll -= delta.y;
        }

        // Handle Wheel Scroll
        if content_response.hovered() {
            let scroll_delta = ui.input(|i| i.raw_scroll_delta);
            if scroll_delta.y != 0.0 {
                if ui.input(|i| i.modifiers.ctrl) {
                    // Zoom time? Or Vertical Scroll?
                    // Usually wheel is vertical scroll. Ctrl+Wheel is zoom.
                    // Let's do vertical scroll for now.
                    *vertical_scroll -= scroll_delta.y;
                    interacted = true;
                } else if ui.input(|i| i.modifiers.shift) {
                    // Horizontal Scroll (Shift + Wheel)
                    let duration = time_range.end() - time_range.start();
                    let dt = -scroll_delta.y / content_rect.width() * duration;
                    let new_start = (*time_range.start() + dt).max(0.0);
                    let new_end = (new_start + duration).min(total_duration);
                    let new_start = (new_end - duration).max(0.0);
                    *time_range = new_start..=new_end;
                    interacted = true;
                } else {
                    *vertical_scroll -= scroll_delta.y;
                    interacted = true;
                }
            }
            if scroll_delta.x != 0.0 {
                let duration = time_range.end() - time_range.start();
                let dt = -scroll_delta.x / content_rect.width() * duration;
                let new_start = (*time_range.start() + dt).max(0.0);
                let new_end = (new_start + duration).min(total_duration);
                let new_start = (new_end - duration).max(0.0);
                *time_range = new_start..=new_end;
                interacted = true;
            }
        }

        // Clamp Vertical Scroll
        let max_scroll = (content_height - content_rect.height()).max(0.0);
        *vertical_scroll = vertical_scroll.clamp(0.0, max_scroll);

        let visible_y_range = *vertical_scroll..=(*vertical_scroll + content_rect.height());

        // Draw Left Panel
        {
            ui.allocate_new_ui(
                egui::UiBuilder::new()
                    .max_rect(left_panel_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(left_panel_rect);
                    draw_left_panel(FlLeftPanelContext {
                        ui,
                        rect: left_panel_rect,
                        visible_y: visible_y_range.clone(),
                    });
                },
            );
        }

        // Draw Content
        {
            ui.allocate_new_ui(
                egui::UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
                |ui| {
                    ui.set_clip_rect(content_rect);
                    draw_content(FlContentContext {
                        ui,
                        rect: content_rect,
                        visible_time: time_range.clone(),
                        visible_y: visible_y_range,
                    });
                },
            );
        }
        (interacted, seek_to)
    }

    fn zoom_scroll_bar(
        ui: &mut Ui,
        rect: Rect,
        time_range: &mut RangeInclusive<f32>,
        total_duration: f32,
    ) -> bool {
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, Color32::from_gray(30));

        let start = *time_range.start();
        let end = *time_range.end();
        let duration = end - start;
        let mut changed = false;

        let map_x = |t: f32| rect.left() + (t / total_duration) * rect.width();
        let unmap_x = |x: f32| (x - rect.left()) / rect.width() * total_duration;

        let mut bar_left = map_x(start);
        let mut bar_right = map_x(end);

        // Ensure minimum visual width to prevent handle overlap
        let min_bar_width = 16.0;
        if bar_right - bar_left < min_bar_width {
            let center = (bar_left + bar_right) / 2.0;
            bar_left = center - min_bar_width / 2.0;
            bar_right = center + min_bar_width / 2.0;
        }

        // Clamp to view area
        if bar_left < rect.left() {
            bar_left = rect.left();
            bar_right = (bar_left + min_bar_width).max(bar_right);
        }
        if bar_right > rect.right() {
            bar_right = rect.right();
            bar_left = (bar_right - min_bar_width).min(bar_left);
        }

        let bar_rect =
            Rect::from_min_max(pos2(bar_left, rect.top()), pos2(bar_right, rect.bottom()));

        let interact_id = ui.id().with("scrollbar");
        let response = ui.interact(rect, interact_id, Sense::click_and_drag());

        #[derive(Clone, Copy, Debug, PartialEq)]
        enum DragMode {
            None,
            Pan,
            ResizeLeft,
            ResizeRight,
        }

        let mode_id = interact_id.with("mode");
        let mut mode = ui.data(|d| d.get_temp(mode_id)).unwrap_or(DragMode::None);

        if mode == DragMode::ResizeLeft || mode == DragMode::ResizeRight {
            ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
        } else if response.hovered() {
            if let Some(hover_pos) = response.hover_pos() {
                let edge_dist = 10.0;
                if (hover_pos.x - bar_left).abs() < edge_dist
                    || (hover_pos.x - bar_right).abs() < edge_dist
                {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                }
            }
        }

        if response.drag_started() {
            let start_pos = response.interact_pointer_pos().unwrap();
            let edge_dist = 10.0; // Larger hit area
            if (start_pos.x - bar_left).abs() < edge_dist {
                mode = DragMode::ResizeLeft;
            } else if (start_pos.x - bar_right).abs() < edge_dist {
                mode = DragMode::ResizeRight;
            } else if bar_rect.contains(start_pos) {
                mode = DragMode::Pan;
            } else {
                mode = DragMode::Pan;
                // Jump to click
                let click_t = unmap_x(start_pos.x);
                let half_dur = duration / 2.0;
                let new_start = (click_t - half_dur).max(0.0);
                let new_end = (new_start + duration).min(total_duration);
                let new_start = (new_end - duration).max(0.0);
                *time_range = new_start..=new_end;
                changed = true;
            }
            ui.data_mut(|d| d.insert_temp(mode_id, mode));
        }

        if response.drag_stopped() {
            ui.data_mut(|d| d.insert_temp(mode_id, DragMode::None));
        }

        if response.dragged() {
            let delta_x = response.drag_delta().x;
            let delta_t = delta_x / rect.width() * total_duration;

            match mode {
                DragMode::Pan => {
                    let new_start = (start + delta_t).max(0.0);
                    let new_end = (new_start + duration).min(total_duration);
                    let new_start = (new_end - duration).max(0.0);
                    *time_range = new_start..=new_end;
                    changed = true;
                }
                DragMode::ResizeLeft => {
                    let new_start = (start + delta_t).min(end - 0.1).max(0.0);
                    *time_range = new_start..=end;
                    changed = true;
                }
                DragMode::ResizeRight => {
                    let new_end = (end + delta_t).max(start + 0.1).min(total_duration);
                    *time_range = start..=new_end;
                    changed = true;
                }
                DragMode::None => {}
            }
        }

        // Visuals
        let color = if response.hovered() || response.dragged() {
            Color32::from_gray(150)
        } else {
            Color32::from_gray(100)
        };
        painter.rect_filled(bar_rect, 4.0, color);
        painter.add(egui::Shape::rect_stroke(
            bar_rect,
            4.0,
            Stroke::new(1.0, Color32::WHITE),
            StrokeKind::Middle,
        ));

        // Draw handles hints
        // Left
        painter.line_segment(
            [
                pos2(bar_left + 4.0, rect.center().y - 4.0),
                pos2(bar_left + 4.0, rect.center().y + 4.0),
            ],
            Stroke::new(2.0, Color32::BLACK),
        );
        // Right
        painter.line_segment(
            [
                pos2(bar_right - 4.0, rect.center().y - 4.0),
                pos2(bar_right - 4.0, rect.center().y + 4.0),
            ],
            Stroke::new(2.0, Color32::BLACK),
        );
        changed
    }
}
