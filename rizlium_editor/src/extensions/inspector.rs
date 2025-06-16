use bevy::prelude::*;
use egui::{ScrollArea, Ui};
use rizlium_chart::{
    chart::Chart,
    editing::{
        chart_path::{ChartPath, LinePath, LinePointPath},
        NotePath,
    },
};
use rizlium_render::GameChart;
use rust_i18n::t;

use helium_framework::prelude::*;

use crate::RizliumDockStateMirror;

use super::editing::{world_view::cam_response::WorldMouseEvent, ChartEditHistory};

#[derive(Resource, Default)]
pub struct SelectedItem {
    pub item: Option<ChartItem>,
}

pub enum ChartItem {
    LinePoint(LinePointPath),
    Line(LinePath),
    Note(NotePath),
}

pub struct Inspector;

impl Plugin for Inspector {
    fn build(&self, app: &mut App) {
        app.register_tab(
            "inspector",
            t!("inspector.tab"),
            logs,
            resource_exists::<GameChart>,
        )
        .init_resource::<SelectedItem>();
        app.register_tab(
            "debugger",
            t!("debugger.tab"),
            debug_window,
            resource_exists::<ChartEditHistory>,
        );
    }
}

fn logs(InMut(mut ui): InMut<Ui>, chart: Res<GameChart>, selected: Res<SelectedItem>) {
    let Some(ref item) = selected.item else {
        ui.weak(t!("tab.logs.select_to_inspect"));
        return;
    };
    let ui = &mut ui;
    if let ChartItem::LinePoint(l) = item {
        show_ui(ui, *l, &chart, |ui, line_point| {
            ui.columns(2, |columns| {
                columns[0].label("easing:");
                columns[1].label(format!("{:?}", line_point.ease_type));
                columns[0].label("time:");
                columns[1].label(line_point.time.to_string());
                columns[0].label("canvas:");
                columns[1].label(line_point.relevant.canvas.to_string());
            });
        })
    }
}

fn show_ui<P: ChartPath>(
    ui: &mut Ui,
    item_path: P,
    chart: &Chart,
    show: impl FnOnce(&mut Ui, &P::Out),
) {
    match item_path.get(chart) {
        Ok(item) => show(ui, item),
        Err(err) => {
            ui.colored_label(egui::Color32::RED, err.to_string());
        }
    };
}

fn bevy_inspector(InMut(ui): InMut<Ui>, world: &mut World) {
    // bevy_inspector_egui::bevy_inspector::ui_for_world(world, &mut ui);
}

fn debug_window(
    InMut(ui): InMut<Ui>,
    // history: Res<ChartEditHistory>,
    // mut event: EventReader<WorldMouseEvent>,
    mirror: Res<RizliumDockStateMirror>,
) {
    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        // ui.heading("cast_result");
        // ui.label(format!("{:?}", event.read().next()));
        // for it in history.history_descriptions() {
        //     ui.label(it.clone());
        // }
        // ui.heading("Preedits");
        // for ed in history.preedit_datas() {
        //     ui.label(format!("{:#?}", ed.inverse()));
        // }
        ui.code_editor(&mut format!("{mirror:?}"))
    });
}
