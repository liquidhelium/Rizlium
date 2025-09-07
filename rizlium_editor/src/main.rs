// 引入 Bevy 的全局错误处理器，用于捕获和记录未处理的系统错误。
use bevy::ecs::error::GLOBAL_ERROR_HANDLER;
// 引入 Rune 脚本语言支持插件。
use rizlium_editor::rune_extensions::HeliumRuneSupportPlugin;
// 引入编辑器扩展的聚合插件。
use rizlium_editor::extensions::ExtensionsPlugin;
// 引入编辑器设置管理插件。
use rizlium_editor::settings_module::SettingsPlugin;
// 引入编辑器音频控制插件。
use rizlium_editor::time_and_audio::EditorAudioPlugin;
// 从 `rizlium_editor` 库中引入多个核心组件和插件。
use rizlium_editor::{
    sync_dock_state, // 用于同步停靠状态的系统
    MainUIPlugin, // 主 UI 插件
    RizliumDockState, // 停靠状态的数据结构
    RizliumDockStateMirror, // 停靠状态的镜像，用于检测变化
    WindowUpdateControlPlugin // 窗口更新控制插件
};


// 引入 Bevy 的核心预设模块。
use bevy::prelude::*;
// 引入 `bevy_egui` 插件，用于集成 egui UI 框架。
use bevy_egui::EguiPlugin;
// 引入 `bevy_persistent` 插件的预设模块，用于数据持久化。
use bevy_persistent::prelude::*;
// 从 `rizlium_editor` 库中引入其他组件和插件。
use rizlium_editor::{
    CountFpsPlugin, // FPS 计数器插件
    EditorState, // 编辑器全局状态资源
    project::RecentFiles, // 最近文件列表的数据结构
    RizTabPresets, // 标签页布局预设
};
// 引入项目管理插件和状态。
use rizlium_editor::project::{ProjectPlugin, ProjectState};
// 引入 `rizlium` 的渲染插件。
use rizlium_render::RizliumRenderingPlugin;

/// `main` 函数是整个程序的入口点。
fn main() {
    // 设置一个全局错误处理器。这是一个回调函数，当 Bevy 的 ECS 系统中发生任何未捕获的错误时，
    // 就会调用这个回调，将错误信息记录下来。这对于调试非常有用。
    GLOBAL_ERROR_HANDLER.set(|err, ctx| {
        // 使用 `error!` 宏记录详细的错误上下文和错误信息。
        error!("Encountered an error! \n ========= \n Context:\n{ctx:#?}\n ========= \n Error:\n {err:#?}")
    }).expect("Cannot set global error handler. It has been set by a library?"); // 如果设置失败，则 panic。

    // 创建一个新的 Bevy 应用实例。
    App::new()
        // 使用 `add_plugins` 方法一次性添加所有需要的插件。
        .add_plugins((
            // `DefaultPlugins` 包含了一组 Bevy 官方推荐的基础插件，如窗口管理、输入、渲染等。
            DefaultPlugins.build(),
            // 添加 `EguiPlugin`，用于在 Bevy 应用中渲染 egui 界面。
            EguiPlugin {
                enable_multipass_for_primary_context: false
            },
            // 添加自定义的 `helium_framework` 核心插件。
            helium_framework::HeliumFramework,
            // 添加 FPS 计数器插件。
            CountFpsPlugin,
            // 添加窗口更新控制插件，用于在非播放状态下降低 CPU 使用率。
            WindowUpdateControlPlugin,
            // 添加设置管理插件。
            SettingsPlugin,
            // 添加所有编辑器扩展的聚合插件。
            ExtensionsPlugin,
            // 添加编辑器音频插件。
            EditorAudioPlugin,
            // 添加项目管理插件。
            ProjectPlugin,
            // 添加主 UI 插件。
            MainUIPlugin,
            // 添加 `rizlium` 渲染插件，并指定它使用 `ProjectState` 作为谱面数据的提供者。
            RizliumRenderingPlugin::<ProjectState>::default(),
            // 添加 Rune 脚本支持插件。
            HeliumRuneSupportPlugin
        ))
        // 初始化 `EditorState` 资源，如果它不存在的话。
        .init_resource::<EditorState>()
        // 插入 `RizliumDockStateMirror` 资源，用于辅助检测停靠状态的变化。
        .insert_resource(RizliumDockStateMirror::default())
        // 在应用启动（Startup）阶段运行 `setup_persistent` 系统，用于设置持久化资源。
        .add_systems(Startup, setup_persistent)
        // 在每次更新前（PreUpdate）运行 `sync_dock_state` 系统。
        // `.run_if` 条件确保该系统仅在 `RizliumDockState` 或其镜像资源发生变化时才运行，以提高性能。
        .add_systems(
            PreUpdate,
            sync_dock_state.run_if(
                resource_changed::<Persistent<RizliumDockState>>
                    .or(resource_changed::<RizliumDockStateMirror>),
            ),
        )
        // 在每次更新后（PostUpdate）也运行 `sync_dock_state` 系统，以确保状态完全同步。
        .add_systems(
            PostUpdate,
            sync_dock_state.run_if(
                resource_changed::<Persistent<RizliumDockState>>
                    .or(resource_changed::<RizliumDockStateMirror>),
            ),
        )
        // 在每次更新前运行 `persist_dock_state` 系统，用于在应用退出时保存停靠状态。
        .add_systems(PreUpdate, persist_dock_state)
        // 启动 Bevy 应用的主循环。
        .run();
}

/// `setup_persistent` 是一个 Bevy 系统，在应用启动时运行，负责配置所有需要持久化的资源。
fn setup_persistent(mut commands: Commands) {
    // 获取用户操作系统的标准配置目录路径。
    let config_dir = dirs::config_dir()
        .expect("Config dir is None") // 如果找不到配置目录，则 panic。
        .join("rizlium-editor"); // 在配置目录下创建一个名为 "rizlium-editor" 的子目录。

    // 配置并插入 `RizTabPresets` 的持久化资源。
    commands.insert_resource(
        Persistent::<RizTabPresets>::builder()
            .format(StorageFormat::Json) // 使用 JSON 格式存储。
            .name("Tab layout presets") // 为该资源指定一个可读的名称。
            .path(config_dir.join("layout-presets.json")) // 指定文件的保存路径。
            .default(RizTabPresets::default()) // 如果文件不存在，则使用默认值。
            .build() // 构建 `Persistent` 资源。
            .expect("failed to setup tab presets"), // 如果构建失败，则 panic。
    );
    // 配置并插入 `RecentFiles` 的持久化资源。
    commands.insert_resource(
        Persistent::<RecentFiles>::builder()
            .format(StorageFormat::Json) // 使用 JSON 格式。
            .name("Recent files")
            .path(config_dir.join("recent-files.json"))
            .default(RecentFiles::default())
            .build()
            .expect("failed to setup recent files"),
    );
    // 配置并插入 `RizliumDockState` 的持久化资源。
    commands.insert_resource(
        Persistent::<RizliumDockState>::builder()
            .format(StorageFormat::Toml) // 使用 TOML 格式，更适合存储复杂的结构化数据。
            .name("Dock state")
            .path(config_dir.join("dock-state.toml"))
            .default(RizliumDockState::default())
            .build()
            .expect("failed to setup dock state"),
    );
}

/// `persist_dock_state` 是一个 Bevy 系统，用于在应用退出时保存停靠布局的状态。
fn persist_dock_state(
    // `EventReader` 用于读取事件，这里我们关心 `AppExit` 事件。
    mut events: EventReader<bevy::app::AppExit>,
    // 可变地访问 `RizliumDockState` 的持久化资源。
    mut state: ResMut<Persistent<RizliumDockState>>,
) -> Result<()> { // 使用 `anyhow::Result` 进行简单的错误处理。
    // 检查是否有 `AppExit` 事件发生。`AppExit` 事件在应用即将关闭时由 Bevy 发送。
    if !events.is_empty() {
        // 如果有退出事件，记录一条调试信息。
        debug!("AppExit event received, persisting dock state.");
        // 调用 `persist` 方法将当前的停靠状态写入到之前配置文件中指定的路径。
        state.persist()?;
    }
    // 返回成功。
    Ok(())
}
