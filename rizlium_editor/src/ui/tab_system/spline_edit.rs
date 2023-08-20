
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rizlium_render::{GameChart, GameTime};

use crate::{
    ui::editing::{spline_editor_horizontal, timeline_horizontal},
    TabProvider,
};

#[derive(SystemParam)]
pub struct SplineWindow<'w, 's> {
    chart: Res<'w, GameChart>,
    current: Local<'s, usize>,
    cache_range: Local<'s, (f32, f32)>,
    scale: Local<'s, [f32;2]>,
    time: Res<'w, GameTime>,
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
            mut scale,
            time,
        } = state.get(world);
        if *scale == [0.,0.] {
            *scale = [200.,200.];
        }
        let view = ui.available_rect_before_wrap();
        timeline_horizontal(
            ui,
            **time,
            &mut (cache_range.0..=cache_range.1),
            &mut scale[0],
            view,
        );
        let mut show_first = false;
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = 500.;

            show_first |= ui.add(egui::Slider::new(
                &mut *current,
                0..=(chart.lines.len() - 1),
            )).changed();
            ui.add(egui::Slider::new(&mut scale[0], 1.0..=2000.0).logarithmic(true));
            ui.add(egui::Slider::new(&mut scale[1], 1.0..=2000.0).logarithmic(true));
        });
        show_first |= ui.button("view").clicked();

        let spline = &chart.lines[*current].points;
        let response = spline_editor_horizontal(
            ui,
            spline,
            &mut Some(0),
            **time,
            &mut scale,
            &mut (0.0..=1000.0),
            show_first
        );
        let range = response.view_data;
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
