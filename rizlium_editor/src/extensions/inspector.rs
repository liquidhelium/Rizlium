use bevy::prelude::*;
use egui::{Label, Ui};
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
        app.register_tab("inspector", t!("inspector.tab"), logs, resource_exists::<GameChart>)
            .init_resource::<SelectedItem>();
    }
}

fn logs(In(mut ui): In<Ui>, chart: Res<GameChart>, selected: Res<SelectedItem>) {
    let Some(ref item) = selected.item else {
        ui.weak(t!("tab.logs.select_to_inspect"));
        return;
    };
    let ui = &mut ui;
    match item {
        ChartItem::LinePoint(l) => show_ui(ui, l.clone(), &chart, |ui, line_point| {
            ui.columns(2, |columns| {
                columns[0].label("easing:");
                columns[1].label(format!("{:?}", line_point.ease_type));
                columns[0].label("time:");
                columns[1].label(line_point.time.to_string());
                columns[0].label("canvas:");
                columns[1].label(line_point.relevant.canvas.to_string());
            });
        }),
        _ => (),
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

fn bevy_inspector(In(mut ui): In<Ui>, world: &mut World) {
    // bevy_inspector_egui::bevy_inspector::ui_for_world(world, &mut ui);
}
