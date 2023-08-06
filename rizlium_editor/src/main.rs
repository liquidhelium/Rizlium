use std::collections::HashMap;
use std::fmt::format;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages};
use bevy::window::{PresentMode, PrimaryWindow, RequestRedraw};
use bevy::winit::WinitSettings;
use bevy::{prelude::*, render::render_resource::TextureDescriptor};
use bevy_egui::EguiContexts;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_persistent::prelude::*;
use egui::{FontData, FontDefinitions};
use egui_dock::DockArea;
use rizlium_editor::{
    dock_window_menu_button, EditorState, RizDockTree, RizTabPresets, RizTabViewer, RizTabs,
};
use rizlium_render::{GameChart, GameTime, GameView, RizliumRenderingPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            EguiPlugin,
            RizliumRenderingPlugin {
                config: (),
                init_with_chart: None,
            },
            LogDiagnosticsPlugin::default(),
            FrameTimeDiagnosticsPlugin,
        ))
        .insert_resource(Msaa::Sample4)
        .init_resource::<EditorState>()
        .init_resource::<RizDockTree>()
        .init_resource::<RizTabs>()
        .add_systems(
            PreStartup,
            (setup_game_view /* egui_font */,).after(bevy_egui::EguiStartupSet::InitContexts),
        )
        .add_systems(Startup, (change_render_type, setup_tab_presets))
        .add_systems(Update, egui_render)
        .add_systems(
            PostUpdate,
            update_type_changing.run_if(resource_changed::<GameTime>()),
        )
        .insert_resource(WinitSettings::desktop_app())
        .run();
}
fn change_render_type(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    window.single_mut().present_mode = PresentMode::AutoNoVsync;
}

fn update_type_changing(mut event: EventWriter<RequestRedraw>) {
    event.send(RequestRedraw);
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
    commands.insert_resource(GameView(image_handle.clone()));
}

fn setup_tab_presets(mut commands: Commands) {
    let config_dir = dirs::config_dir()
        .expect("Config dir is None")
        .join("rizlium-editor");
    commands.insert_resource(
        Persistent::<RizTabPresets>::builder()
            .format(StorageFormat::Json)
            .name("Tab layout presets")
            .path(config_dir.join("layout-presets.json"))
            .default(RizTabPresets {
                presets: HashMap::new(),
            })
            .build()
            .expect("failed to setup tab presets"),
    )
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
    drop(binding);
    drop(egui_context);
    let mut editor_state = world
        .remove_resource::<EditorState>()
        .expect("EditorState does not exist");
    ctx.set_debug_on_hover(editor_state.debug_resources.show_cursor);
    let mut tree = world
        .remove_resource::<RizDockTree>()
        .expect("RizDockTree does not exist");
    let mut tab = world.remove_resource::<RizTabs>().unwrap();
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
    world.resource_scope(|_world, mut presets: Mut<Persistent<RizTabPresets>>| {
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Rizlium");
                ui.toggle_value(
                    &mut editor_state.debug_resources.show_cursor,
                    "Show cursor (Debug)",
                );
                dock_window_menu_button(ui, "View", &mut tree.tree, &tab.tabs);
                ui.menu_button("Presets", |ui| {
                    for (key, preset_tree) in presets.get().presets.iter() {
                        if ui.button(key).clicked() {
                            tree.tree = preset_tree.clone();
                        }
                    }
                    if ui.button("Save current as preset").clicked() {
                        presets
                            .update(|presets| {
                                presets.presets.insert("New".into(), tree.tree.clone());
                            })
                            .unwrap();
                    }
                });
            });
        });
    });

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
            ui.centered_and_justified(|ui| ui.heading("Rizlium\n(Dev version)"));
        });
    }
    world.insert_resource(editor_state);
    world.insert_resource(tree);
    world.insert_resource(tab);
}
