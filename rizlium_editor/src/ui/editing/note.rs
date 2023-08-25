use egui::{scroll_area::ScrollAreaOutput, Color32, NumExt, Ui};
use rizlium_chart::prelude::*;

pub fn note_editor_vertical(
    ui: &mut Ui,
    focus: Option<usize>,
    lanes: &[&[Note]],
    scale: &mut f32,
    scroll_to_first: bool,
    row_width: f32,
    max_time: f32,
) {
    assert_ne!(*scale, 0.);
    egui::ScrollArea::both()
        .id_source("note_editor_inner_v")
        .auto_shrink([true; 2])
        .show_cols(ui, row_width, lanes.len(), |ui, range| {
            // left timeline
            
            for lane in &lanes[range] {
                let (_, rect) = ui.allocate_space([row_width, max_time * *scale].into());
                for note in *lane {
                    ui.painter().circle_stroke(
                        [rect.center().x, rect.bottom() - note.time * *scale].into(),
                        4.,
                        egui::Stroke::new(5., Color32::GREEN),
                    );
                }
                ui.ctx()
                    .debug_painter()
                    .debug_rect(rect, Color32::DEBUG_COLOR, "note");
            }
        });
}

trait ScrollAreaExt {
    fn show_cols<R>(
        self,
        ui: &mut Ui,
        col_width_sans_spacing: f32,
        total_cols: usize,
        add_contents: impl FnOnce(&mut Ui, std::ops::Range<usize>) -> R,
    ) -> ScrollAreaOutput<R>;
}

impl ScrollAreaExt for egui::ScrollArea {
    fn show_cols<R>(
        self,
        ui: &mut Ui,
        col_width_sans_spacing: f32,
        total_cols: usize,
        add_contents: impl FnOnce(&mut Ui, std::ops::Range<usize>) -> R,
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
                    .horizontal(|viewport_ui| {
                        viewport_ui.skip_ahead_auto_ids(min_col); // Make sure we get consistent IDs.
                        add_contents(viewport_ui, min_col..max_col)
                    })
                    .inner
            })
            .inner
        })
    }
}
