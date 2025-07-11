#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use bevy::{
    diagnostic::FrameCount,
    prelude::*,
    window::{PresentMode, PrimaryWindow, RequestRedraw},
};
use egui::{Color32, Rect, RichText, Ui, UiBuilder};
// use egui_tracing::EventCollector;
use rizlium_render::GameTime;
i18n!();

use rust_i18n::i18n;
pub use ui::*;
mod chart_loader;
mod editor_actions;
pub mod extensions;
mod files;
pub mod settings_module;
pub mod utils;
pub use chart_loader::{ChartLoadingEvent, ChartLoadingPlugin, LoadChartEvent};
pub use editor_actions::*;
pub use files::*;
pub mod extra_window_control;
mod ui;
#[derive(Debug, Resource, Default)]
pub struct EditorState {
    pub debug_resources: DebugResources,
    pub editing_presets: bool,
    pub is_editing_text: bool,
}
#[derive(Debug, Default)]
pub struct DebugResources {
    pub show_cursor: bool,
}

// #[derive(Resource)]
// pub struct EventCollectorResource(pub EventCollector);

pub struct CountFpsPlugin;

impl Plugin for CountFpsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NowFps>()
            .add_systems(PostUpdate, compute_fps);
    }
}

#[derive(Resource, Default)]
pub struct NowFps(pub u32);

#[derive(Deref, DerefMut)]
struct SecondTimer(Timer);
impl Default for SecondTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1), TimerMode::Repeating))
    }
}

fn compute_fps(
    mut last_fps: Local<u32>,
    current: Res<FrameCount>,
    mut fps: ResMut<NowFps>,
    time: Res<Time>,
    mut fps_timer: Local<SecondTimer>,
) {
    if fps_timer.tick(time.delta()).finished() {
        fps.0 = current.0 - *last_fps;
        *last_fps = current.0;
    }
}

pub fn ui_when_no_dock(ui: &mut Ui, recents: &RecentFiles, commands: &mut ManualEditorCommands) {
    let main_rect = ui.available_rect_before_wrap().shrink(50.);
    ui.allocate_new_ui(UiBuilder::new().max_rect(main_rect), |ui: &mut Ui| {
        let center_rect = if main_rect.width() >= 500. {
            Rect::from_center_size(main_rect.center(), [500., main_rect.height()].into())
        } else {
            main_rect
        };
        ui.allocate_new_ui(UiBuilder::new().max_rect(center_rect), |ui: &mut Ui| {
            ui.label(RichText::new("Rizlium").size(50.).color(Color32::WHITE));
            ui.weak(
                RichText::new("Just an editor")
                    .size(20.)
                    .color(Color32::GRAY),
            );
            let center_rect = ui.available_rect_before_wrap();
            let max_rect = left_half(&center_rect);
            ui.allocate_new_ui(UiBuilder::new().max_rect(max_rect), |ui: &mut Ui| {
                ui.heading("Open");
                for recent in recents.iter().rev() {
                    if ui.link(recent).clicked() {
                        commands.load_chart(recent.clone());
                    }
                }
                ui.heading("Heading 2");
                ui.label("a?");
                ui.label("a?");
            });
            let max_rect = right_half(&center_rect);
            ui.allocate_new_ui(UiBuilder::new().max_rect(max_rect), |ui: &mut Ui| {
                ui.label(do114514::<100>());
            });
        });
    });
}

fn left_half(rect: &Rect) -> Rect {
    Rect::from_min_size(rect.min, [rect.width() / 2., rect.height()].into())
}

fn right_half(rect: &Rect) -> Rect {
    Rect::from_min_max(
        [rect.max.x - rect.width() / 2., rect.min.y].into(),
        rect.max,
    )
}

fn do114514<const LEN: usize>() -> String {
    ["114514"; LEN].join("")
}

pub struct WindowUpdateControlPlugin;

impl Plugin for WindowUpdateControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, change_render_type).add_systems(
            PostUpdate,
            update_type_changing.run_if(resource_changed::<GameTime>),
        );
    }
}

fn change_render_type(mut window: Query<&mut Window, With<PrimaryWindow>>) -> Result<()> {
    window
        .single_mut()
        .map(|mut a| a.present_mode = PresentMode::AutoNoVsync)?;
    Ok(())
}

fn update_type_changing(mut event: EventWriter<RequestRedraw>) {
    event.write(RequestRedraw);
}
