use crate::tab_system::TabRegistrationExt;

use self::{note::note_editor_vertical, spline::spline_editor_horizontal};
use bevy::prelude::*;
use egui::Ui;
use rizlium_render::{GameChart, GameTime, TimeControlEvent};

mod note;
mod spline;
mod timeline;
mod world_view;

pub struct Editing;

impl Plugin for Editing {
    fn build(&self, app: &mut App) {
        app.register_tab(
            "editing.note".into(),
            "Notes",
            note_window,
            resource_exists::<GameChart>(),
        )
        .register_tab(
            "editing.spline".into(),
            "Splines",
            spline_edit,
            resource_exists::<GameChart>(),
        );

        app.add_plugins(world_view::WorldViewPlugin);
    }
}

fn note_window(
    In(ui): In<&mut Ui>,
    chart: Res<GameChart>,
    mut focused: Local<usize>,
    mut scale: Local<f32>,
    mut row_width: Local<f32>,
    time: Res<GameTime>,
) {
    if *scale == 0. {
        *scale = 200.;
    }
    if *row_width == 0. {
        *row_width = 50.
    }
    let _view = ui.available_rect_before_wrap();
    let _show_first = false;
    ui.scope(|ui| {
        ui.style_mut().spacing.slider_width = 500.;

        ui.add(egui::Slider::new(
            &mut *focused,
            0..=(chart.lines.len() - 1),
        ));
        ui.add(egui::Slider::new(&mut *scale, 1.0..=2000.0).logarithmic(true));
        ui.add(egui::Slider::new(&mut *row_width, 10.0..=200.0));
    });
    note_editor_vertical(
        ui,
        Some(0),
        chart
            .lines
            .iter()
            .map(|l| l.notes.as_slice())
            .enumerate()
            .collect::<Vec<_>>()
            .as_slice(),
        **time,
        &mut scale,
        *row_width,
        200.,
    )
}

pub fn spline_edit(
    In(ui): In<&mut Ui>,
    chart: Res<GameChart>,
    mut current: Local<usize>,
    mut cache_range: Local<(f32, f32)>,
    mut scale: Local<[f32; 2]>,
    time: Res<GameTime>,
    mut ev: EventWriter<TimeControlEvent>,
) {
    if *scale == [0., 0.] {
        *scale = [200., 200.];
    }
    let mut show_first = false;
    ui.scope(|ui| {
        ui.style_mut().spacing.slider_width = 500.;

        show_first |= ui
            .add(egui::Slider::new(
                &mut *current,
                0..=(chart.lines.len() - 1),
            ))
            .changed();
        ui.add(egui::Slider::new(&mut scale[0], 1.0..=2000.0).logarithmic(true));
        ui.add(egui::Slider::new(&mut scale[1], 1.0..=2000.0).logarithmic(true));
    });
    show_first |= ui.button("view").clicked();
    ui.allocate_ui_at_rect(ui.available_rect_before_wrap(), |ui| {
        let spline = &chart.lines[*current].points;
        let response =
            spline_editor_horizontal(ui, spline, Some(0), **time, &mut scale, show_first);
        if let Some(to) = response.seek_to {
            ev.send(TimeControlEvent::Seek(to));
        }
        let range = response.view_rect;
        cache_range.0 = range.x_range().min;
        cache_range.1 = range.y_range().max;
    });
}
