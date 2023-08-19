use std::ops::RangeInclusive;

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rizlium_render::GameChart;

use crate::{
    ui::editing::{spline_editor_horizontal, timeline},
    TabProvider,
};

#[derive(SystemParam)]
pub struct SplineWindow<'w, 's> {
    chart: Res<'w, GameChart>,
    current: Local<'s, usize>,
    cache_range: Local<'s, (f32, f32)>,
    scale: Local<'s, f32>,
}

impl<'w, 's> TabProvider for SplineWindow<'w, 's> {
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
    ) {
        let SplineWindow {
            chart,
            mut current,
            mut cache_range,
            mut scale
        } = state.get(world);
        if *scale == 0. {
            *scale = 200.
        }
        timeline(ui, 0., &mut (cache_range.0..=cache_range.1), &mut scale);
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = 500.;

            ui.add(egui::Slider::new(
                &mut *current,
                0..=(chart.lines.len() - 1),
            ));
            ui.add(egui::Slider::new(&mut *scale, 1.0..=2000.0))
        });

        let spline = &chart.lines[*current].points;
        let range =
            spline_editor_horizontal(ui, spline, &mut Some(0), 0., &mut scale, &mut (0.0..=1000.0))
                .view_data;
        cache_range.0 = *range.time_range.start();
        cache_range.1 = *range.time_range.end();
    }
    fn name() -> String {
        "Spline".into()
    }
    fn avaliable(world: &World) -> bool {
        world.contains_resource::<GameChart>()
    }
}
