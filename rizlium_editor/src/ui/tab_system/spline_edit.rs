
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rizlium_render::{GameChart, GameTime, TimeControlEvent, GameChartCache};

use crate::{
    ui::editing::spline_editor_horizontal,
    TabProvider, EditorCommands,
};

#[derive(SystemParam)]
pub struct SplineWindow<'w, 's> {
    chart: Res<'w, GameChart>,
    cache: Res<'w, GameChartCache>,
    current: Local<'s, usize>,
    cache_range: Local<'s, (f32, f32)>,
    scale: Local<'s, [f32;2]>,
    time: Res<'w, GameTime>,
    editor_commands: EditorCommands<'s>,
}

impl<'w, 's> TabProvider for SplineWindow<'w, 's> {
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
    ) {
        let SplineWindow {
            chart,
            cache,
            mut current,
            mut cache_range,
            mut scale,
            time,
            mut editor_commands
        } = state.get(world);
        if *scale == [0.,0.] {
            *scale = [200.,200.];
        }
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
        ui.allocate_ui_at_rect(ui.available_rect_before_wrap(), |ui| {
            let spline = &chart.lines[*current].points;
            let response = spline_editor_horizontal(
                ui,
                spline,
                Some(0),
                **time,
                &mut scale,
                show_first
            );
            if let Some(to) = response.seek_to {
                editor_commands.time_control(TimeControlEvent::Seek(cache.remap_beat(to)));
            }
            let range = response.view_rect;
            cache_range.0 = *range.x_range().start();
            cache_range.1 = *range.y_range().end();
        });
        
    }
    fn name() -> String {
        "Spline".into()
    }
    fn avaliable(world: &World) -> bool {
        world.contains_resource::<GameChart>()
    }
}
