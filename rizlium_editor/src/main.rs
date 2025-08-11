use bevy::ecs::error::GLOBAL_ERROR_HANDLER;
// use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use rizlium_editor::extensions::ExtensionsPlugin;
use rizlium_editor::settings_module::SettingsPlugin;
use rizlium_editor::time_and_audio::EditorAudioPlugin;
use rizlium_editor::{
    sync_dock_state, MainUIPlugin, RizliumDockState, RizliumDockStateMirror, WindowUpdateControlPlugin
};


use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use bevy_persistent::prelude::*;
use rizlium_editor::{
 CountFpsPlugin, EditorState, RecentFiles,
    RizTabPresets,
};
use rizlium_editor::project::{ProjectPlugin, ProjectState};
use rizlium_render::RizliumRenderingPlugin;

fn main() {
    GLOBAL_ERROR_HANDLER.set(|err, ctx| {
        error!("Encountered an error! \n ========= \n Context:\n{ctx:#?}\n ========= \n Error:\n {err:#?}")
    }).expect("Cannot set global error handler. It has been set by a library?");
    App::new()
        .add_plugins((
            DefaultPlugins.build(),
            EguiPlugin {
                enable_multipass_for_primary_context: false
            },
            helium_framework::HeliumFramework,
            CountFpsPlugin,
            WindowUpdateControlPlugin,
            SettingsPlugin,
            ExtensionsPlugin,
            EditorAudioPlugin,
            ProjectPlugin,
            MainUIPlugin,
            RizliumRenderingPlugin::<ProjectState>::default(),
        ))
        .init_resource::<EditorState>()
        .insert_resource(RizliumDockStateMirror::default())
        .add_systems(Startup, setup_persistent)
        .add_systems(
            PreUpdate,
            sync_dock_state.run_if(
                resource_changed::<Persistent<RizliumDockState>>
                    .or(resource_changed::<RizliumDockStateMirror>),
            ),
        )
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
}

fn persist_dock_state(
    events: EventReader<bevy::app::AppExit>,
    state: ResMut<Persistent<RizliumDockState>>,
) -> Result<()> {
    if !events.is_empty() {
        debug!("{events:?}");
        state.persist()?;
    }
    Ok(())
}
