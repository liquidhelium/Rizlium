use egui::{Pos2, Rect, Ui};
use strum::IntoEnumIterator;

use super::world_view::tools::Tool;

pub fn tool_select_bar(ui: &mut Ui, origin: Pos2, curr_tool: &mut Tool, area: Rect) {
    egui::Area::new(ui.id().with("tool select bar"))
        .fixed_pos(origin)
        .movable(false)
        .order(egui::Order::Foreground)
        .constrain_to(area)
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
