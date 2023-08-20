use egui::Ui;
use rizlium_chart::prelude::*;


pub fn note_editor_vertical<'a>(ui: &mut Ui, focus: Option<usize>, notes: impl Iterator<Item = &'a [Note]>, scale: &mut f32, scroll_to_first: bool, row_width: f32) {
    assert_ne!(*scale, 0.);
    egui::ScrollArea::horizontal()
        .id_source("note_editor_v")
        .auto_shrink([false;2])
        .show(ui, |ui| {
            let (_, rect) = ui.allocate_space([row_width, ui.available_height()].into());
        });
}