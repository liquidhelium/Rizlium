use egui::{scroll_area::ScrollAreaOutput, Color32, FontId, NumExt, Ui};
use rizlium_chart::prelude::*;

use super::timeline::timeline_vertical;

pub fn note_editor_vertical(
    ui: &mut Ui,
    _focus: Option<usize>,
    lanes: &[(impl ToString, &[Note])],
    cursor: f32,
    scale: &mut f32,
    row_width: f32,
    max_time: f32,
) {
    assert_ne!(*scale, 0.);
    let editor_area = {
        let mut r = ui.available_rect_before_wrap();
        r.max -= egui::Vec2::splat(ui.spacing().scroll.bar_width);
        r
    };
    let timeline_area = egui::Rect::from_x_y_ranges(
        editor_area.left()..=editor_area.left() + 20.,
        editor_area.bottom()..=editor_area.top(),
    );
    let max_y = max_time * *scale;
    egui::ScrollArea::both()
        .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysVisible)
        .id_source("note_editor_inner_v")
        .auto_shrink([false; 2])
        .stick_to_bottom(true)
        .show_cols_with_viewport(ui, row_width, lanes.len(), |ui, range, view| {
            let range_y = view.y_range();
            let time_start = (max_y - range_y.max) / *scale;
            let time_end = (max_y - range_y.min) / *scale;
            for (label, lane) in &lanes[range] {
                let (_, rect) = ui.allocate_space([row_width, max_y].into());
                ui.painter().rect_filled(rect, 0., Color32::BLACK);
                ui.painter().text(
                    [rect.center().x, editor_area.bottom()].into(),
                    egui::Align2::CENTER_BOTTOM,
                    label.to_string(),
                    FontId::default(),
                    Color32::WHITE,
                );
                for note in *lane {
                    ui.painter().circle_stroke(
                        [rect.center().x, rect.bottom() - note.time * *scale].into(),
                        4.,
                        egui::Stroke::new(5., Color32::GREEN),
                    );
                }
            }
            // left timeline is shown after lanes to prevent overlapping.
            timeline_vertical(
                ui,
                cursor,
                &mut (time_start..=time_end),
                scale,
                editor_area,
                timeline_area,
            );
        });
}

trait ScrollAreaExt {
    fn show_cols_with_viewport<R>(
        self,
        ui: &mut Ui,
        col_width_sans_spacing: f32,
        total_cols: usize,
        add_contents: impl FnOnce(&mut Ui, std::ops::Range<usize>, egui::Rect) -> R,
    ) -> ScrollAreaOutput<R>;
}

impl ScrollAreaExt for egui::ScrollArea {
    fn show_cols_with_viewport<R>(
        self,
        ui: &mut Ui,
        col_width_sans_spacing: f32,
        total_cols: usize,
        add_contents: impl FnOnce(&mut Ui, std::ops::Range<usize>, egui::Rect) -> R,
    ) -> ScrollAreaOutput<R> {
        let spacing = ui.spacing().item_spacing;
        let col_width_with_spacing = col_width_sans_spacing + spacing.x;
        self.show_viewport(ui, |ui, viewport| {
            ui.set_width((col_width_with_spacing * total_cols as f32 - spacing.x).at_least(0.0));

            let mut min_col = (viewport.min.x / col_width_with_spacing).floor() as usize;
            let mut max_col = (viewport.max.x / col_width_with_spacing).ceil() as usize + 1;
            if max_col > total_cols {
                let diff = max_col.saturating_sub(min_col);
                max_col = total_cols;
                min_col = total_cols.saturating_sub(diff);
            }

            let x_min = ui.max_rect().left() + min_col as f32 * col_width_with_spacing;
            let x_max = ui.max_rect().left() + max_col as f32 * col_width_with_spacing;

            let rect = egui::Rect::from_x_y_ranges(x_min..=x_max, ui.max_rect().y_range());

            ui.allocate_ui_at_rect(rect, |viewport_ui| {
                viewport_ui
                    .horizontal_top(|viewport_ui| {
                        viewport_ui.skip_ahead_auto_ids(min_col); // Make sure we get consistent IDs.
                        add_contents(viewport_ui, min_col..max_col, viewport)
                    })
                    .inner
            })
            .inner
        })
    }
}
