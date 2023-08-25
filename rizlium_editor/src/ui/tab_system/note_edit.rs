


use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rizlium_render::{GameChart, GameTime};

use crate::{
    ui::editing::note_editor_vertical,
    TabProvider,
};

#[derive(SystemParam)]
pub struct NoteWindow<'w, 's> {
    chart: Res<'w, GameChart>,
    current: Local<'s, usize>,
    scale: Local<'s, f32>,
    row_width: Local<'s, f32>,
    time: Res<'w, GameTime>,
}

impl<'w, 's> TabProvider for NoteWindow<'w, 's> {
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
    ) {
        let NoteWindow {
            chart,
            current: mut focused,
            mut scale,
            mut row_width,
            time,
        } = state.get(world);
        if *scale == 0. {
            *scale = 200.;
        }
        if *row_width == 0. {
            *row_width = 50.
        }
        let view = ui.available_rect_before_wrap();
        let mut show_first = false;
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = 500.;
            
            ui.add(egui::Slider::new(
                &mut *focused,
                0..=(chart.lines.len() - 1),
            ));
            ui.add(egui::Slider::new(&mut *scale, 1.0..=2000.0).logarithmic(true));
            ui.add(egui::Slider::new(&mut *row_width, 10.0..=200.0));
        });
        note_editor_vertical(ui, Some(0), chart.lines.iter().map(|l| l.notes.as_slice()).collect::<Vec<_>>().as_slice(), &mut scale, false, *row_width, 200.)
        
        
    }
    fn name() -> String {
        "Notes".into()
    }
    fn avaliable(world: &World) -> bool {
        world.contains_resource::<GameChart>()
    }
}
