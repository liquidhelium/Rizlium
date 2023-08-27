
use bevy::render::view::WindowRenderPlugin;
use rizlium_editor::hotkeys::HotkeyPlugin;
use rizlium_editor::widgets::{widget, widget_with, DockButtons, LayoutPresetEdit, PresetButtons, RecentButtons};
use rizlium_editor::WindowUpdateControlPlugin;

use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::{PresentMode, PrimaryWindow, RequestRedraw};
use bevy::winit::WinitSettings;
use bevy::{prelude::*, render::render_resource::TextureDescriptor};
use bevy_egui::EguiContexts;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_persistent::prelude::*;
use egui::{Align2, FontData, FontDefinitions, Layout};
use egui_dock::DockArea;
use rizlium_editor::{
    open_chart, ui_when_no_dock, CountFpsPlugin, EditorState, NowFps, PendingDialog, RecentFiles, RizDockTree, RizTabPresets,
    RizTabViewer, RizTabs, ManualEditorCommands,
};
use rizlium_render::{GameChart, GameView, RizliumRenderingPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            RizliumRenderingPlugin {
                config: (),
                init_with_chart: None,
            },
            // WorldInspectorPlugin::default(),
            CountFpsPlugin,
            HotkeyPlugin,
            WindowUpdateControlPlugin
        ))
        .insert_resource(Msaa::Sample4)
        .init_resource::<EditorState>()
        .init_resource::<RizDockTree>()
        .init_resource::<RizTabs>()
        .init_resource::<PendingDialog>()
        .init_resource::<RecentFiles>()
        .add_systems(
            PreStartup,
            (setup_game_view /* egui_font */,).after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, setup_persistent)
        .add_systems(Update, egui_render)
        .add_systems(
            PostUpdate,
                open_chart,
        )
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

fn egui_font(mut egui_context: EguiContexts) {
    // TODO: this font name is hard coded
    let data = font_kit::source::SystemSource::new()
        .select_by_postscript_name("SourceHanSansCN-Normal")
        .unwrap()
        .load()
        .unwrap()
        .copy_font_data()
        .unwrap();
    let mut def = FontDefinitions::default();
    def.font_data
        .insert("cn".into(), FontData::from_owned(Vec::clone(&data)));
    def.families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, "cn".to_owned());
    let ctx = egui_context.ctx_mut();
    ctx.set_fonts(def);
}
fn egui_render(world: &mut World) {
    let mut egui_context = world.query::<(&mut EguiContext, With<PrimaryWindow>)>();
    let mut binding = egui_context.single_mut(world).0;
    let ctx = &binding.get_mut().clone();
    let mut editor_state = world
        .remove_resource::<EditorState>()
        .expect("EditorState does not exist");
    ctx.set_debug_on_hover(editor_state.debug_resources.show_cursor);
    let mut commands = ManualEditorCommands::default();
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
            ui.label("Rizlium");
            ui.toggle_value(
                &mut editor_state.debug_resources.show_cursor,
                "Show cursor (Debug)",
            );
            ui.menu_button("File", |ui| {
                if ui.button("Open..").clicked() {
                    commands.open_dialog_and_load_chart();
                }
                ui.separator();
                ui.weak("Recent");
                widget::<RecentButtons>(world, ui);
            });
            ui.menu_button("View", |ui| {
                ui.menu_button("Presets", |ui| {
                    widget_with::<PresetButtons>(world, ui, &mut editor_state.editing_presets);
                });
                widget::<DockButtons>(world, ui);
            });
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                world.resource_scope(|_world, fps: Mut<'_, NowFps>| {
                    ui.label(format!("fps: {}", fps.0));
                });
            });
        });
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
        world.resource_scope(|world: &mut World, mut tree: Mut<'_, RizDockTree>| {
            DockArea::new(&mut tree.tree)
                .scroll_area_in_tabs(false)
                .show(
                    ctx,
                    &mut RizTabViewer {
                        world,
                        editor_state: &mut editor_state,
                        tabs: &mut tab.tabs,
                    },
                );

            if tree.tree.is_empty() {
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui_when_no_dock(
                        ui,
                        world.resource::<Persistent<RecentFiles>>(),
                        &mut commands,
                    );
                });
            }
        });
    });
    commands.apply_manual(world);
    world.insert_resource(editor_state);
}
