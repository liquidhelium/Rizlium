use bevy::log::{Level, LogPlugin};
use rizlium_editor::extensions::command_panel::CommandPanelImpl;
use rizlium_editor::extensions::{EditorMenuEntrys, ExtensionsPlugin};
use rizlium_editor::extra_window_control::{DragWindowRequested, ExtraWindowControlPlugin};
use rizlium_editor::hotkeys::HotkeyPlugin;
use rizlium_editor::notification::NotificationPlugin;
use rizlium_editor::tab_system::{FocusedTab, TabPlugin, TabRegistry};
use rizlium_editor::widgets::{widget, LayoutPresetEdit};
use rizlium_editor::{
    ActionPlugin, EventCollectorResource, FilePlugin, RizTabViewerNext, WindowUpdateControlPlugin,
};

use bevy::window::PrimaryWindow;

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_persistent::prelude::*;
use egui::{Align2, Layout};
use egui_dock::DockArea;
use rizlium_editor::{
    ui_when_no_dock, CountFpsPlugin, EditorState, ManualEditorCommands, NowFps, RecentFiles,
    RizDockState, RizTabPresets,
};
use rizlium_render::{GameChart, RizliumRenderingPlugin};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

fn main() {
    // let collector = egui_tracing::EventCollector::default().with_level(Level::DEBUG);
    // let default_filter = { format!("{},{}", Level::DEBUG, "wgpu=error,naga=warn") };
    //     let filter_layer = EnvFilter::try_from_default_env()
    //         .or_else(|_| EnvFilter::try_new(&default_filter))
    //         .unwrap();
    // tracing_subscriber::registry().with(filter_layer).with(collector.clone()).with(tracing_subscriber::fmt::Layer::default()).init();
    App::new()
        .add_plugins((
            DefaultPlugins.build()/* .disable::<LogPlugin>() */,
            EguiPlugin,
            RizliumRenderingPlugin {
                config: (),
                init_with_chart: None,
            },
            NotificationPlugin,
            CountFpsPlugin,
            WindowUpdateControlPlugin,
            ActionPlugin,
            HotkeyPlugin,
            TabPlugin,
            FilePlugin,
            ExtensionsPlugin,
            ExtraWindowControlPlugin,
        ))
        .insert_resource(Msaa::Sample4)
        .init_resource::<EditorState>()
        .init_resource::<RizDockState>()
        .add_event::<DragWindowRequested>()
        // .insert_resource(EventCollectorResource(collector))
        .add_systems(Startup, setup_persistent)
        .add_systems(Update, egui_render)
        .run();
}

fn setup_persistent(mut commands: Commands) {
    let config_dir = dirs::config_dir()
        .expect("Config dir is None")
        .join("rizlium-editor");
    commands.insert_resource(
        Persistent::<RizTabPresets>::builder()
            .format(StorageFormat::Json)
            .name("Tab layout presets")
            .path(config_dir.join("layout-presets.json"))
            .default(RizTabPresets::default())
            .build()
            .expect("failed to setup tab presets"),
    );
    commands.insert_resource(
        Persistent::<RecentFiles>::builder()
            .format(StorageFormat::Json)
            .name("Recent files")
            .path(config_dir.join("recent-files.json"))
            .default(RecentFiles::default())
            .build()
            .expect("failed to setup recent files"),
    );
}
fn egui_render(world: &mut World) {
    let mut egui_context = world.query::<(&mut EguiContext, With<PrimaryWindow>)>();
    let mut binding = egui_context.single_mut(world).0;
    let ctx = &binding.get_mut().clone();
    let mut editor_state = world
        .remove_resource::<EditorState>()
        .expect("EditorState does not exist");
    // ctx.set_debug_on_hover(editor_state.debug_resources.show_cursor);
    let mut commands = ManualEditorCommands::default();
    let _window = world.query_filtered::<Entity, With<Window>>().single(world);
    // todo: status into extension
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            if world.contains_resource::<GameChart>() {
                let chart = world.resource::<GameChart>();
                ui.label("Ready");
                ui.separator();
                ui.label(format!("{} segments", chart.segment_count()));
                ui.separator();
                ui.label(format!("{} notes", chart.note_count()));
            } else {
                ui.label("No chart loaded");
            }
        });
    });
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            world.resource_scope(|world: &mut World, entries: Mut<EditorMenuEntrys>| {
                entries.foreach_ui(ui, world);
            });
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                world.resource_scope(|_world, fps: Mut<'_, NowFps>| {
                    ui.label(format!("fps: {}", fps.0));
                });
            });
        });
        widget::<CommandPanelImpl>(world, ui);
    });
    let before = editor_state.editing_presets;
    egui::Window::new("Presets")
        .collapsible(false)
        .open(&mut editor_state.editing_presets)
        .anchor(Align2::CENTER_CENTER, [0.; 2])
        .show(ctx, |ui| {
            widget::<LayoutPresetEdit>(world, ui);
        });
    if before != editor_state.editing_presets {
        commands.persist_resource::<RizTabPresets>();
    }
    world.resource_scope(|world: &mut World, mut registry: Mut<'_, TabRegistry>| {
        world.resource_scope(|world: &mut World, mut state: Mut<'_, RizDockState>| {
            if state.state.main_surface().is_empty() {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui_when_no_dock(
                        ui,
                        world.resource::<Persistent<RecentFiles>>(),
                        &mut commands,
                    );
                });
            }
            DockArea::new(&mut state.state).show(
                ctx,
                &mut RizTabViewerNext {
                    registry: &mut registry,
                    world,
                },
            );
            world.resource_mut::<FocusedTab>().0 = state.state.find_active_focused().unzip().1.cloned(); // todo: move this into proper file
        });
    });
    editor_state.is_editing_text = ctx.output(|out| out.mutable_text_under_cursor);

    commands.apply_manual(world);
    world.insert_resource(editor_state);
}
