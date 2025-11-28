use self::{note::note_editor_vertical, tool_config_window::tool_config};
use crate::{project::ProjectState, MainMenuContext};
use bevy::prelude::*;
use egui::{emath::RectTransform, vec2, Color32, Sense, Stroke, Ui, UiBuilder};
use helium_framework::prelude::*;
use rizlium_chart::{
    chart::Spline,
    editing::{ChartCommand, ChartCommands, EditHistory},
};
use rizlium_render::{ChartProvider, GameTime};
use rust_i18n::t;
use spline::SplineView;
pub mod note;
pub mod spline;
pub mod timeline;
mod tool_config_window;
mod tool_select_bar;
mod undo_redo;
pub mod world_view;
pub struct Editing;

impl Plugin for Editing {
    fn build(&self, app: &mut App) {
        app.register_tab(
            "edit.note",
            t!("edit.note.tab"),
            note_window,
            ProjectState::has_chart_system(),
        )
        .register_tab(
            "edit.spline",
            t!("edit.spline.tab"),
            spline_edit,
            ProjectState::has_chart_system(),
        )
        .register_tab(
            "edit.tool_config",
            t!("edit.tool_config.tab"),
            tool_config,
            ProjectState::has_chart_system(),
        );
        app.add_plugins((world_view::WorldViewPlugin, timeline::TimelinePlugin))
            .init_resource::<ChartEditHistory>();
        app.reflect_system("edit.undo", t!("edit.undo.desc"), undo_redo::undo)
            .reflect_system("edit.redo", t!("edit.redo.desc"), undo_redo::redo)
            .reflect_system(
                "edit.push_edit_command",
                "Push an edit command to history",
                push_edit_command,
            );
        use KeyCode::*;
        app.register_hotkey("edit.undo", [Hotkey::new_global([ControlLeft, KeyZ])])
            .register_hotkey("edit.redo", [Hotkey::new_global([ControlLeft, KeyY])])
            .register_submenu::<MainMenuContext>("edit", t!("edit.name"))
            .register_widget::<MainMenuContext>(
                "edit/historylist",
                "History",
                "edit.history.widget",
            )
            .register_command::<MainMenuContext>("edit/undo", t!("edit.undo.name"), "edit.undo")
            .register_command::<MainMenuContext>("edit/redo", t!("edit.redo.name"), "edit.redo")
            .reflect_system("edit.history.widget", "history widget", history_list);
    }
}

fn history_list(
    InMut(ui): InMut<Ui>,
    chart: Res<ProjectState>,
    chart_edit_history: Res<ChartEditHistory>,
) {
    egui::ScrollArea::vertical()
        .auto_shrink([false, true])
        .show_viewport(ui, |ui, _rect| {
            for (_, entry) in chart_edit_history
                .0
                .history_descriptions()
                .iter()
                .enumerate()
            {
                ui.label(&**entry);
            }
        });
}

fn push_edit_command(
    In(command): In<ChartCommands>,
    mut chart_edit_history: ResMut<ChartEditHistory>,
    mut chart: ResMut<ProjectState>,
    mut toast: ResMut<ToastsStorage>,
) {
    if let Err(err) = chart_edit_history.push(command, &mut chart.chart_mut()) {
        toast.error(err.to_string());
    }
}
#[derive(Deref, DerefMut, Resource, Default)]
pub struct ChartEditHistory(EditHistory);
fn note_window(
    InMut(ui): InMut<Ui>,
    chart: Res<ProjectState>,
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
    ui.scope(|ui| {
        ui.style_mut().spacing.slider_width = 500.;
        ui.add(egui::Slider::new(
            &mut *focused,
            0..=(chart.chart().lines.len() - 1),
        ));
        ui.add(egui::Slider::new(&mut *scale, 1.0..=2000.0).logarithmic(true));
        ui.add(egui::Slider::new(&mut *row_width, 10.0..=200.0));
    });
    note_editor_vertical(
        ui,
        Some(0),
        chart
            .chart()
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
    InMut(ui): InMut<Ui>,
    chart: Res<ProjectState>,
    mut current: Local<usize>,
    mut visible_rect: Local<Option<egui::Rect>>,
    _external: Local<Spline<f32>>,
) {
    let mut show_first = false;
    ui.scope(|ui| {
        ui.style_mut().spacing.slider_width = 500.;

        show_first |= ui
            .add(egui::Slider::new(
                &mut *current,
                0..=(chart.chart().canvases.len() - 1),
            ))
            .changed();
    });
    let (res, spline_view) = {
        let max_rect = ui.available_rect_before_wrap();

        ui.allocate_new_ui(UiBuilder::new().max_rect(max_rect), |ui| {
            let spline = &chart.chart().canvases[*current].speed;
            let spline_view =
                SplineView::new(ui, spline, *visible_rect, spline::Orientation::Horizontal);
            let (response, _) = spline_view.ui(ui);
            let spline_area = spline_view.spline_area();
            const WIDTH: f32 = 80.0;
            const RATIO: f32 = 9. / 16.;
            let indicating_rect_full = egui::Rect::from_min_size(
                response.rect.min + vec2(20., 20.),
                vec2(WIDTH, WIDTH * RATIO),
            );
            let spline_to_interact = RectTransform::from_to(spline_area, indicating_rect_full);
            let indicating_rect_inner =
                spline_to_interact.transform_rect(spline_view.visible_spline_area());
            ui.painter_at(response.rect).rect(
                indicating_rect_full,
                0.,
                Color32::from_white_alpha(20),
                Stroke::new(1., Color32::BLACK),
                egui::StrokeKind::Middle,
            );
            let mut alpha = 20;
            let inner_interact = ui.interact(
                indicating_rect_inner,
                ui.id().with("indicating_rect_inner"),
                Sense::drag(),
            );
            if inner_interact.hovered() {
                alpha += 10;
            }
            ui.painter_at(response.rect).rect_filled(
                indicating_rect_inner,
                0.,
                Color32::from_white_alpha(alpha),
            );
            if inner_interact.dragged() {
                let transformed = spline_to_interact
                    .inverse()
                    .transform_rect(indicating_rect_inner.translate(inner_interact.drag_delta()));
                *visible_rect = Some(transformed);
            }

            (response, spline_view)
        })
    }
    .inner;
    if res.dragged() {
        let scale = spline_view.view_to_spline().scale();
        let delta = (-res.drag_delta()) * scale;
        let rect = visible_rect.unwrap_or(spline_view.visible_spline_area());
        *visible_rect = Some(rect.translate(delta));
    }
    if show_first {
        *visible_rect = None;
    }
}
