use bevy::prelude::*;
use bevy::ecs::system::SystemParam;
use egui::{Slider, Layout};
use rizlium_render::{GameChart, GameChartCache};

use crate::TabProvider;

#[derive(SystemParam)]
pub struct CanvasWindow<'w,'s> {
    chart: Res<'w,GameChart>,
    cache: Res<'w, GameChartCache>,
    current_canvas: Local<'s,usize>,
}

impl TabProvider for CanvasWindow<'_,'_> {
    fn system(world: &mut World, state: &mut bevy::ecs::system::SystemState<Self>, ui: &mut egui::Ui) {
        let CanvasWindow::<'_,'_> {
            chart,
            cache,
            mut current_canvas,
        } = state.get_mut(world);
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = 500.;
            ui.add(Slider::new(&mut *current_canvas, 0..=chart.canvases.len()));
        });
        // ui.allocate_space(ui.available_size());
        ui.allocate_ui_with_layout(ui.available_size(),Layout::left_to_right(egui::Align::Min),|ui| {

            egui::ScrollArea::new([false, true]).id_source("id_source1").auto_shrink([true,false]).show(ui, |ui| {
                ui.text_edit_multiline(&mut format!("{:#?}", chart.canvases.get(*current_canvas)));
            });
            egui::ScrollArea::new([false, true]).id_source("id_source2").auto_shrink([true,false]).show(ui, |ui| {
                ui.text_edit_multiline(&mut format!("{:#?}", cache.canvas_y.get(*current_canvas)));
            });
        });
    }
    fn name() -> String {
        "Canvas inspector".into()
    }
    fn avaliable(world: &World) -> bool {
        world.contains_resource::<GameChart>() && world.contains_resource::<GameChartCache>()
    }
}