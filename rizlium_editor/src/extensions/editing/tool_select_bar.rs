use egui::{vec2, Color32, Pos2, Rect, Ui};
use strum::IntoEnumIterator;

use super::world_view::tools::Tool;

pub fn tool_select_bar(ui: &mut Ui, origin: Pos2, curr_tool: &mut Tool) {
    ui.ctx().debug_painter().debug_rect(
        Rect::from_center_size(origin, vec2(2., 2.)),
        Color32::BLUE,
        "origin",
    );
    egui::Area::new(ui.id().with("tool select bar"))
        .fixed_pos(origin)
        .movable(false)
        .show(ui.ctx(), |ui| {
            egui::Frame::menu(ui.style()).show(ui, |ui| {
                ui.set_max_width(40.);
                ui.vertical_centered(|ui| {
                    for tool in Tool::iter() {
                        if ui
                            .selectable_label(tool == *curr_tool, format!("{tool:?}"))
                            .clicked()
                        {
                            *curr_tool = tool;
                        };
                    }
                });
            });
        });
}
