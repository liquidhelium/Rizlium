use bevy::ecs::system::RunSystemOnce;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use helium_framework::menu::EditorMenuEntrys;
use helium_framework::prelude::HeTabViewer;
use helium_framework::tab_system::{FocusedTab, TabRegistry};
use helium_framework::widgets::widget;
use rizlium_editor::extensions::command_panel::command_panel;
use rizlium_editor::extensions::ExtensionsPlugin;
use rizlium_editor::extra_window_control::{DragWindowRequested, ExtraWindowControlPlugin};
use rizlium_editor::settings_module::SettingsPlugin;
use rizlium_editor::time_and_audio::EditorAudioPlugin;
use rizlium_editor::{
    sync_dock_state, ui_when_no_dock, ChartLoadingPlugin, FilePlugin, RizliumDockState, RizliumDockStateMirror, WindowUpdateControlPlugin
};

use bevy::window::PrimaryWindow;

use bevy::prelude::*;
use bevy_egui::{EguiContext, EguiPlugin};
use bevy_persistent::prelude::*;
use egui::{Color32, FontData, FontDefinitions, FontFamily, Layout, Stroke, Visuals};
use egui_dock::{DockArea, TabInteractionStyle};
use rizlium_editor::{
 CountFpsPlugin, EditorState, ManualEditorCommands, NowFps, RecentFiles,
    RizTabPresets,
};
use rizlium_editor::project::{ProjectPlugin, ProjectState};
use rizlium_render::{ChartProvider, RizliumRenderingPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build(),
            EguiPlugin {
                enable_multipass_for_primary_context: false,
            },
            DefaultInspectorConfigPlugin,
            RizliumRenderingPlugin::<ProjectState>::default(),
            helium_framework::HeliumFramework,
            CountFpsPlugin,
            WindowUpdateControlPlugin,
            FilePlugin,
            SettingsPlugin,
            ChartLoadingPlugin,
            ExtensionsPlugin,
            ExtraWindowControlPlugin,
            EditorAudioPlugin,
            ProjectPlugin

        ))
        .init_resource::<EditorState>()
        .insert_resource(RizliumDockStateMirror::default())
        .add_event::<DragWindowRequested>()
        // .insert_resource(EventCollectorResource(collector))
        .add_systems(Startup, (setup_persistent, setup_font))
        .add_systems(
            PreUpdate,
            sync_dock_state.run_if(
                resource_changed::<Persistent<RizliumDockState>>
                    .or(resource_changed::<RizliumDockStateMirror>),
            ),
        )
        .add_systems(Update, egui_render)
        .add_systems(
            PostUpdate,
            sync_dock_state.run_if(
                resource_changed::<Persistent<RizliumDockState>>
                    .or(resource_changed::<RizliumDockStateMirror>),
            ),
        )
        .add_systems(PreUpdate, persist_dock_state)
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
    commands.insert_resource(
        Persistent::<RizliumDockState>::builder()
            .format(StorageFormat::Toml)
            .name("Dock state")
            .path(config_dir.join("dock-state.toml"))
            .default(RizliumDockState::default())
            .build()
            .expect("failed to setup dock state"),
    );
    // commands.spawn((Camera2d, Msaa::Sample4));
}

fn setup_font(mut context: Query<&mut EguiContext>) {
    context.par_iter_mut().for_each(|mut c| {
        debug!("Setting up fonts for egui");
        let mut fonts = FontDefinitions::default();
        fonts.font_data.insert(
            "SourceHanSansSC".to_owned(),
            FontData::from_static(include_bytes!("../assets/SourceHanSansSC.otf")).into(),
        ); // .ttf and .otf supported
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "SourceHanSansSC".to_owned());
        c.get_mut().set_fonts(fonts);
        debug!("Fonts set up successfully");
    });
}

fn egui_render(world: &mut World) -> Result<()> {
    let mut egui_context = world.query_filtered::<&mut EguiContext, With<PrimaryWindow>>();
    let mut binding = egui_context.single_mut(world)?;
    let ctx = &binding.get_mut().clone();
    let mut editor_state = world
        .remove_resource::<EditorState>()
        .expect("EditorState does not exist");
    let mut commands = ManualEditorCommands::default();
    ctx.all_styles_mut(|style| {
        style.visuals.widgets.noninteractive.bg_stroke.width = 0.5;
        style.visuals.window_corner_radius = 0.0.into();
    });
    // todo: status into extension
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            if world.run_system_once(ProjectState::has_chart_system()).is_ok_and(|ok| ok) {
                let chart = world.resource::<ProjectState>();
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
            world.resource_scope(|world: &mut World, mut entries: Mut<EditorMenuEntrys>| {
                ui.style_mut().visuals = Visuals {
                    widgets: egui::style::Widgets {
                        inactive: egui::style::WidgetVisuals {
                            weak_bg_fill: rgba(50, 50, 50, 0.0),
                            bg_stroke: egui::Stroke::new(0.0, rgba(200, 200, 200, 0.0)),

                            ..Visuals::dark().widgets.inactive
                        },
                        hovered: egui::style::WidgetVisuals {
                            weak_bg_fill: rgba(100, 100, 100, 0.4),
                            bg_stroke: egui::Stroke::new(1.0, rgba(200, 200, 200, 0.0)),

                            ..Visuals::dark().widgets.hovered
                        },
                        ..Default::default()
                    },
                    ..Visuals::dark()
                };
                entries.foreach_ui(ui, world);
            });
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                world.resource_scope(|_world, fps: Mut<'_, NowFps>| {
                    ui.label(format!("fps: {}", fps.0));
                });
            });
        });
        widget(world, ui, command_panel);
    });
    world.resource_scope(|world: &mut World, mut registry: Mut<'_, TabRegistry>| {
        world.resource_scope(
            |world: &mut World, mut state: Mut<'_, Persistent<RizliumDockState>>| {
                if state.0.main_surface().is_empty() {
                    egui::CentralPanel::default().show(ctx, |ui| {
                        widget(world, ui, ui_when_no_dock);
                    });
                }
                DockArea::new(&mut state.0)
                    .style(egui_dock::Style {
                        separator: egui_dock::SeparatorStyle {
                            width: 0.5,
                            ..Default::default()
                        },
                        main_surface_border_rounding: 0.0.into(),
                        tab_bar: egui_dock::TabBarStyle {
                            corner_radius: 0.0.into(),
                            ..egui_dock::Style::from_egui(&ctx.style()).tab_bar
                        },
                        tab: egui_dock::TabStyle {
                            active: TabInteractionStyle {
                                outline_color: rgba(0, 0, 0, 0.0),
                                corner_radius: 0.0.into(),
                                ..egui_dock::Style::from_egui(&ctx.style()).tab.active
                            },
                            focused: TabInteractionStyle {
                                outline_color: rgba(0, 0, 0, 0.0),
                                corner_radius: 0.0.into(),
                                ..egui_dock::Style::from_egui(&ctx.style()).tab.focused
                            },
                            hovered: TabInteractionStyle {
                                outline_color: rgba(0, 0, 0, 0.0),
                                bg_fill: rgba(37, 37, 37,1.0),
                                corner_radius: 0.0.into(),
                                ..egui_dock::Style::from_egui(&ctx.style()).tab.hovered
                            },
                            inactive: TabInteractionStyle {
                                outline_color: rgba(0, 0, 0, 0.0),
                                corner_radius: 0.0.into(),
                                ..egui_dock::Style::from_egui(&ctx.style()).tab.inactive
                            },
                            ..egui_dock::Style::from_egui(&ctx.style()).tab
                        },
                        ..egui_dock::Style::from_egui(&ctx.style())
                    })
                    .show(
                        ctx,
                        &mut HeTabViewer {
                            registry: &mut registry,
                            world,
                        },
                    );
                world.resource_mut::<FocusedTab>().0 =
                    state.0.find_active_focused().unzip().1.cloned();
                // todo: move this into proper file
            },
        );
    });
    editor_state.is_editing_text = ctx.output(|out| out.mutable_text_under_cursor);

    commands.apply_manual(world);
    world.insert_resource(editor_state);
    Ok(())
}

fn rgba(arg_1: i32, arg_2: i32, arg_3: i32, arg_4: f64) -> egui::Color32 {
    Color32::from_rgba_unmultiplied(arg_1 as u8, arg_2 as u8, arg_3 as u8, (arg_4 * 255.0) as u8)
}
fn persist_dock_state(
    mut events: EventReader<bevy::app::AppExit>,
    state: ResMut<Persistent<RizliumDockState>>,
) -> Result<()> {
    if !events.is_empty() {
        debug!("{events:?}");
        state.persist()?;
    }
    Ok(())
}
