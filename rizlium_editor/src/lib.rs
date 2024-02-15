use std::time::Duration;

use bevy::{
    core::FrameCount,
    prelude::*,
    
    window::{PresentMode, PrimaryWindow, RequestRedraw},
    winit::WinitSettings,
};
use egui::{Color32, Rect, RichText, Ui};
use egui_dock::DockState;
use egui_tracing::EventCollector;
use rizlium_render::GameTime;

use ui::tab_system::TabId;
pub use ui::*;
mod editor_actions;
pub mod hotkeys;
mod files;
pub mod extensions;
pub mod utils;
pub mod notification;
pub use files::*;
pub use editor_actions::*;
mod ui;
pub mod extra_window_control;
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

#[derive(Resource)]
pub struct EventCollectorResource(pub EventCollector);

#[derive(Debug, Resource)]
pub struct RizDockState {
    pub state: DockState<TabId>,
}

impl Default for RizDockState {
    fn default() -> Self {
        Self {
            state: DockState::new(vec!["game.view".into()]),
        }
    }
}

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
    ui.allocate_ui_at_rect(main_rect, |ui| {
        let center_rect = if main_rect.width() >= 500. {
            Rect::from_center_size(main_rect.center(), [500., main_rect.height()].into())
        } else {
            main_rect
        };
        ui.allocate_ui_at_rect(center_rect, |ui| {
            ui.label(RichText::new("Rizlium").size(50.).color(Color32::WHITE));
            ui.weak(
                RichText::new("Just an editor")
                    .size(20.)
                    .color(Color32::GRAY),
            );
            let center_rect = ui.available_rect_before_wrap();
            ui.allocate_ui_at_rect(left_half(&center_rect), |ui| {
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
            ui.allocate_ui_at_rect(right_half(&center_rect), |ui| {
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
        app.add_systems(Startup, change_render_type)
            .add_systems(
                PostUpdate,
                update_type_changing.run_if(resource_changed::<GameTime>()),
            )
            .insert_resource(WinitSettings::desktop_app());
    }
}

fn change_render_type(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    window.single_mut().present_mode = PresentMode::AutoNoVsync;
}

fn update_type_changing(mut event: EventWriter<RequestRedraw>) {
    event.send(RequestRedraw);
}
