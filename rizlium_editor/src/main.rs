use bevy::log::LogPlugin;
use rizlium_editor::extensions::command_panel::CommandPanelImpl;
use rizlium_editor::extensions::{EditorMenuEntrys, ExtensionsPlugin};
use rizlium_editor::extra_window_control::{DragWindowRequested, ExtraWindowControlPlugin};
use rizlium_editor::hotkeys::HotkeyPlugin;
use rizlium_editor::widgets::{widget, LayoutPresetEdit};
use rizlium_editor::{ActionPlugin, FilePlugin, InitRizTabsExt, WindowUpdateControlPlugin};

use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::PrimaryWindow;

use bevy::{prelude::*, render::render_resource::TextureDescriptor};
use bevy_egui::EguiContexts;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_persistent::prelude::*;
use egui::{Align2, Layout};
use egui_dock::DockArea;
use rizlium_editor::{
    ui_when_no_dock, CountFpsPlugin, EditorState, ManualEditorCommands, NowFps, RecentFiles,
    RizDockState, RizTabPresets, RizTabViewer, RizTabs,
};
use rizlium_render::{GameChart, GameView, RizliumRenderingPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(LogPlugin {
                level: bevy::log::Level::DEBUG,
                ..Default::default()
            }),
            EguiPlugin,
            RizliumRenderingPlugin {
                config: (),
                init_with_chart: None,
            },
            // WorldInspectorPlugin::default(),
            CountFpsPlugin,
            WindowUpdateControlPlugin,
            ActionPlugin,
            HotkeyPlugin,
            FilePlugin,
            ExtensionsPlugin,
            ExtraWindowControlPlugin,
        ))
        .insert_resource(Msaa::Sample4)
        .init_resource::<EditorState>()
        .init_resource::<RizDockState>()
        .add_event::<DragWindowRequested>()
        .init_riztabs()
        .add_systems(
            PreStartup,
            (setup_game_view, setup_window).after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, setup_persistent)
        .add_systems(Update, egui_render)
        .run();
}
fn setup_game_view(
    mut commands: Commands,
    mut egui_context: EguiContexts,
    mut images: ResMut<Assets<Image>>,
) {
    let size = Extent3d {
        width: 1080,
        height: 1920,
        ..default()
    };
    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: None,
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    image.resize(size);
    let image_handle = images.add(image);
    egui_context.add_image(image_handle.clone());
    commands.insert_resource(GameView(image_handle));
}

fn setup_window(_windows: Query<&mut Window>) {
    // windows.single_mut().decorations = false;
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
    world.resource_scope(|world: &mut World, mut tab: Mut<'_, RizTabs>| {
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
            let focused_tab = state.state.find_active_focused().unzip().1.copied();
            DockArea::new(&mut state.state).show(
                ctx,
                &mut RizTabViewer {
                    world,
                    editor_state: &mut editor_state,
                    tabs: &mut tab.tabs,
                    focused_tab,
                },
            );

            
        });
    });
    editor_state.is_editing_text = ctx.output(|out| out.mutable_text_under_cursor);

    commands.apply_manual(world);
    world.insert_resource(editor_state);
}
