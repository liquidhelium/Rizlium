use std::time::Duration;

use bevy::{
    core::FrameCount,
    prelude::*,
    tasks::{IoTaskPool, Task}, window::{RequestRedraw, PresentMode, PrimaryWindow}, winit::WinitSettings,
};
use egui::{Color32, Rect, RichText, Ui};
use egui_dock::Tree;
use futures_lite::future;
use indexmap::IndexSet;
use rfd::AsyncFileDialog;
use rizlium_render::GameTime;
use serde::{Deserialize, Serialize};

pub use ui::*;
mod editor_commands;
pub mod hotkeys;
pub use editor_commands::*;
mod ui;
#[derive(Debug, Resource, Default)]
pub struct EditorState {
    pub debug_resources: DebugResources,
    pub editing_presets: bool,
}
#[derive(Debug, Default)]
pub struct DebugResources {
    pub show_cursor: bool,
}

#[derive(Debug, Resource)]
pub struct RizDockTree {
    pub tree: Tree<usize>,
}

impl Default for RizDockTree {
    fn default() -> Self {
        Self {
            tree: Tree::new(vec![0]),
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
    if fps_timer.tick(time.raw_delta()).finished() {
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
                for recent in recents.0.iter().rev() {
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

#[derive(Resource, Default)]
pub struct PendingDialog(Option<Task<Option<String>>>);

pub fn open_dialog(container: &mut PendingDialog) {
    info!("opening chart");
    container.0 = Some(IoTaskPool::get().spawn(async {
        let file = AsyncFileDialog::new()
            .add_filter("Bundled chart file", &["zip"])
            .pick_file()
            .await;

        file.map(|file| file.path().to_string_lossy().into_owned())
    }));
}

pub fn open_chart(mut dialog: ResMut<PendingDialog>, mut editor_command: EditorCommands) {
    if let Some(chart) = dialog
        .0
        .as_mut()
        .and_then(|t| future::block_on(future::poll_once(t)))
    {
        dialog.0.take();
        if let Some(chart) = chart {
            editor_command.load_chart(chart);
        }
    }
}

#[derive(Resource, Serialize, Deserialize, Debug, Deref)]
pub struct RecentFiles(#[deref] IndexSet<String>, usize);

impl Default for RecentFiles {
    fn default() -> Self {
        Self(default(), 4)
    }
}

impl RecentFiles {
    pub fn push(&mut self, name: String) {
        if let (idx, false) = self.0.insert_full(name) {
            let value = self.0.shift_remove_index(idx).unwrap();
            self.0.insert(value);
        }
        if self.0.len() > self.1 {
            self.0.shift_remove_index(0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::RecentFiles;

    #[test]
    fn push() {
        let mut rec = RecentFiles::default();
        for i in 1..5 {
            rec.push(i.to_string());
        }
        assert_eq!(
            "RecentFiles({\"1\", \"2\", \"3\", \"4\"}, 4)".to_string(),
            format!("{:?}", rec)
        );
        rec.push("1".into());
        assert_eq!(
            "RecentFiles({\"2\", \"3\", \"4\", \"1\"}, 4)".to_string(),
            format!("{:?}", rec)
        );
        rec.push("3".into());
        assert_eq!(
            "RecentFiles({\"2\", \"4\", \"1\", \"3\"}, 4)".to_string(),
            format!("{:?}", rec)
        );
    }
}

pub struct WindowUpdateControlPlugin;

impl Plugin for WindowUpdateControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, change_render_type).add_systems(
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
