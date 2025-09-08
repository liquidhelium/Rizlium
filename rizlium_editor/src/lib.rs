// 关闭 clippy 的一些 lint 警告，以保持代码简洁。
// `too_many_arguments`: 允许函数拥有较多数量的参数，在 Bevy 系统中很常见。
#![allow(clippy::too_many_arguments)]
// `type_complexity`: 允许复杂的类型签名，这在 Bevy 查询中也可能出现。
#![allow(clippy::type_complexity)]
// `unused`: 允许存在未使用的代码，可能用于临时禁用或开发中的功能。
#![allow(unused)]

// 引入标准库中的 `Duration`，用于处理时间间隔。
use std::time::Duration;

// 引入 Bevy 引擎的各个模块。
use bevy::{
    // `FrameCount` 资源，用于获取自应用启动以来渲染的总帧数。
    diagnostic::FrameCount,
    // `RunSystemOnce` trait，允许在需要时一次性地运行某个系统。
    ecs::system::RunSystemOnce as _,
    // 引入 Bevy 的核心预设模块，包含最常用的组件、资源和函数。
    prelude::*,
    // `PresentMode` 用于控制窗口的渲染同步模式（如 VSync），`PrimaryWindow` 用于标识主窗口，`RequestRedraw` 用于请求重绘。
    window::{PresentMode, PrimaryWindow, RequestRedraw},
};
// 引入 `bevy_egui` 插件，用于在 Bevy 中集成 egui UI 框架。
use bevy_egui::{EguiContext, EguiUserTextures};
// 引入 `bevy_persistent` 插件，用于轻松地持久化（保存和加载）Bevy 资源。
use bevy_persistent::Persistent;
// 引入 `egui` 库的核心组件，用于构建用户界面。
use egui::{Color32, Frame, Layout, Rect, RichText, Style, Ui, UiBuilder};
// 引入 `egui-dock` 库，用于创建可停靠的窗口区域。
use egui_dock::DockArea;
// 引入自定义的 `helium_framework` 框架。
use helium_framework::{
    // `MenuSystem` 用于管理菜单栏。
    menu_system::MenuSystem,
    // 引入框架的预设模块，包含常用的类型。
    prelude::{FocusedTab, HeTabViewer, HotkeyRegistry, RSystemRegistry, TabRegistry},
    // `Identifier` 用于创建唯一的标识符。
    utils::identifier::Identifier,
    // `widget` 函数，用于在 egui UI 中执行 Bevy 系统。
    widgets::widget,
};
// 引入 `rizlium_render` 渲染库。
use rizlium_render::{ChartProvider as _, GameTime};
// 使用 `rust_i18n` 宏来初始化国际化功能，它会加载 `locales/` 目录下的翻译文件。
i18n!();

// 再次引入 `rust_i18n` 的 `i18n!` 宏，以便在代码中使用。
use rust_i18n::i18n;
// 公开 `ui` 模块的所有内容，使其可以在 crate 外部访问。
pub use ui::*;
// 声明 `editor_actions` 模块，包含编辑器的各种操作命令。
mod editor_actions;
// 声明并公开 `extensions` 模块，管理编辑器的各种扩展插件。
pub mod extensions;
// 声明并公开 `project` 模块，处理项目文件的加载、保存等。
pub mod project;
// 声明并公开 `settings_module` 模块，用于管理编辑器设置。
pub mod settings_module;
// 声明并公开 `time_and_audio` 模块，处理时间和音频的同步与控制。
pub mod time_and_audio;
// 声明并公开 `utils` 模块，提供一些通用工具函数。
pub mod utils;
// 声明并公开 `rune_extensions` 模块，用于集成 Rune 脚本语言。
pub mod rune_extensions;

// 从当前 crate 的子模块中引入所需的类型。
use crate::{
    extensions::command_panel::command_panel, // 命令面板 UI 函数
    project::{LoadChartEvent, ProjectState, RecentFiles}, // 项目相关的事件和状态
    ui::{
        theme::{tab_theme, menu_bar_theme},
        widgets::shortcut_display,
    }, // UI 主题和自定义小部件
};

/// `MainMenuContext` 是一个零大小的结构体，用作主菜单的上下文标识。
/// `helium_framework` 的菜单系统使用这种上下文类型来决定显示哪个菜单。
#[derive(Debug)]
pub struct MainMenuContext;


/// `RightMenuContext` 是一个零大小的结构体，用作顶栏右侧菜单的上下文标识。
#[derive(Debug)]
pub struct RightMenuContext;
// 声明 `ui` 模块，其中包含 UI 相关的代码。
mod ui;
/// `EditorState` 是一个 Bevy 资源，用于存储编辑器的全局状态。
#[derive(Debug, Resource, Default)]
pub struct EditorState {
    // 存储与调试相关的资源状态。
    pub debug_resources: DebugResources,
    // 标记当前是否正在编辑预设。
    pub editing_presets: bool,
    // 标记用户的鼠标光标当前是否位于一个可编辑的文本字段中。
    // 这对于避免在输入文本时触发全局快捷键非常重要。
    pub is_editing_text: bool,
}
/// `DebugResources` 存储了用于调试的各种状态开关。
#[derive(Debug, Default)]
pub struct DebugResources {
    // 是否显示调试光标。
    pub show_cursor: bool,
}

// `icons_def!` 是一个声明宏，用于简化加载图标资源的过程。
// 它可以自动生成一个资源结构体和一个加载图标的 Bevy 系统。
macro_rules! icons_def {
    // 宏的匹配模式：接收结构体名、函数名和一个包含多个图标路径名的元组。
    ($stru:ident, $func:ident, ($($path:ident),+)) => {
        /// 这个结构体将由宏自动生成，用于存储所有图标的 `Handle<Image>`。
        /// Bevy 使用 `Handle` 来异步地、安全地引用资源。
        #[derive(Resource)]
        pub struct $stru {
            // 宏会为每个传入的路径名生成一个字段。
            $($path: Handle<Image>),+
        }

        // 引入 `EguiContexts`，用于将 Bevy 的图像句柄注册到 egui 中。
        use bevy_egui::EguiContexts;
        /// 这个函数也由宏自动生成，它是一个 Bevy 系统，负责在启动时加载所有图标。
        fn load_icons(
            mut commands: Commands, // Bevy 命令，用于插入资源。
            asset_server: Res<AssetServer>, // Bevy 资源服务器，用于加载文件。
            mut egui_context: EguiContexts, // egui 上下文，用于注册图像。
        ) -> Result<()> { // `anyhow::Result` 用于错误处理。
            $(
                // 宏会为每个图标路径生成一行加载代码。
                // `concat!` 和 `stringify!` 用于将标识符转换为字符串路径。
                let $path = asset_server.load(concat!(stringify!($path), ".png"));
            )+
            // 将包含所有图标句柄的结构体作为资源插入到 Bevy `World` 中。
            commands.insert_resource($stru {
                $($path: $path.clone(),)+
            });
            $(
                // 将每个加载的图像添加到 egui 的上下文中，这样 egui 才能渲染它们。
                egui_context.add_image($path);
            )+
            // 表示系统成功执行。
            Ok(())
        }

    };
}

// 使用 `icons_def!` 宏来定义 `Icons` 资源和 `load_icons` 系统。
// 这会自动处理 `assets/` 目录下所有指定 `.png` 文件的加载。
icons_def!(
    Icons,
    load_icons,
    (
        rizlium_colored_64x,
        rizlium_colored,
        rizlium_solid_64x,
        rizlium_solid,
        rizlium_solid_darker
    )
);

/// `CountFpsPlugin` 是一个简单的 Bevy 插件，用于计算并显示当前的 FPS（每秒帧数）。
pub struct CountFpsPlugin;

impl Plugin for CountFpsPlugin {
    fn build(&self, app: &mut App) {
        // 在应用中初始化 `NowFps` 资源，并添加 `compute_fps` 系统。
        app.init_resource::<NowFps>()
            .add_systems(PostUpdate, compute_fps); // `PostUpdate` 确保在所有逻辑更新后运行。
    }
}

/// `NowFps` 资源，用于存储当前计算出的 FPS 值。
#[derive(Resource, Default)]
pub struct NowFps(pub u32);

/// `SecondTimer` 是一个本地计时器，用于每秒触发一次 FPS 计算。
#[derive(Deref, DerefMut)]
struct SecondTimer(Timer);
// 为 `SecondTimer` 实现 `Default` trait，使其默认是一个每秒重复一次的计时器。
impl Default for SecondTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_secs(1), TimerMode::Repeating))
    }
}

/// `compute_fps` 是一个 Bevy 系统，用于计算当前的 FPS。
fn compute_fps(
    mut last_fps: Local<u32>, // `Local` 变量，用于在系统调用之间保持状态（上次的帧数）。
    current: Res<FrameCount>, // 当前的总帧数资源。
    mut fps: ResMut<NowFps>,  // 用于存储结果的 FPS 资源。
    time: Res<Time>,          // Bevy 的时间资源，用于驱动计时器。
    mut fps_timer: Local<SecondTimer>, // 每秒触发一次的计时器。
) {
    // `tick` 方法推进计时器，如果计时器完成了一个周期（即一秒钟过去了），则 `finished()` 返回 true。
    if fps_timer.tick(time.delta()).finished() {
        // 计算自上次更新以来经过的帧数。
        fps.0 = current.0 - *last_fps;
        // 更新 `last_fps` 以备下次计算使用。
        *last_fps = current.0;
    }
}

/// `ui_when_no_dock` 是一个在没有任何停靠窗口时显示的欢迎界面。
/// 它是一个 `widget` 系统，通过 `In` 参数接收 `&mut Ui`。
pub fn ui_when_no_dock(
    In(ui): In<&mut Ui>, // `In` 提取器，用于从输入参数中获取 `&mut Ui`。
    _recents: Res<Persistent<RecentFiles>>, // 最近文件列表资源（当前未使用）。
    mut _events: EventWriter<LoadChartEvent>, // 加载谱面事件的写入器（当前未使用）。
    egui_textures: Res<EguiUserTextures>, // egui 纹理资源，用于获取图像 ID。
    icons: Res<Icons>,   // 图标资源。
    hotkeys: Res<HotkeyRegistry>, // 快捷键注册表资源。
    actions: Res<RSystemRegistry>, // 系统（动作）注册表资源。
) {
    // `desc` 是一个局部闭包，用于显示一个动作的描述。
    let desc = |ui: &mut Ui, id: &str| {
        // 从注册表中查找具有给定 ID 的动作。
        if let Some(action) = actions.get(&Identifier::from(id)) {
            // 使用从右到左的布局，将标签对齐到右侧。
            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {
                // 显示弱化（灰色）的动作描述文本。
                ui.label(RichText::new(action.description.clone()).weak());
            });
        }
    };
    // `keys` 是一个局部闭包，用于显示一个动作的快捷键。
    let keys = |ui: &mut Ui, id: &str| {
        // 从注册表中查找具有给定 ID 的快捷键。
        if let Some(hotkeys) = hotkeys.get(&Identifier::from(id)) {
            // 使用从左到右的布局。
            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                // 重置样式，以确保快捷键显示正常。
                ui.set_style(Style::default());
                let max_index = hotkeys.len() - 1;
                // 遍历所有为该动作注册的快捷键组合。
                for (index, hotkey) in hotkeys.iter().enumerate() {
                    // 使用 `shortcut_display` 自定义小部件来渲染快捷键。
                    shortcut_display(
                        &hotkey
                            .key
                            .iter()
                            .map(|&key| format!("{key:?}"))
                            .collect::<Vec<_>>(),
                        ui,
                    );
                    // 如果不是最后一个快捷键，则添加分号作为分隔符。
                    if index != max_index {
                        ui.label(";");
                    }
                }
            });
        }
    };
    // 获取 Rizlium logo 的 egui 图像 ID。
    let id = egui_textures.image_id(&icons.rizlium_solid_darker).unwrap();
    // 计算主显示区域，并向内收缩 50 像素。
    let main_rect = ui.available_rect_before_wrap().shrink(50.);
    // 在计算出的区域内分配一个新的 UI 空间。
    ui.allocate_ui_at_rect(main_rect, |ui: &mut Ui| {
        // 垂直居中显示 logo。
        ui.vertical_centered(|ui| {
            ui.add(egui::Image::new((id, egui::Vec2::splat(300.))));
        });
        // 再次计算可用区域，并向内收缩 50 像素。
        let main_rect = ui.available_rect_before_wrap().shrink(50.);
        // 创建一个最大宽度为 500 像素的居中矩形，用于显示快捷键提示。
        let center_rect = if main_rect.width() >= 500. {
            Rect::from_center_size(main_rect.center(), [500., main_rect.height()].into())
        } else {
            main_rect
        };
        // 在这个居中矩形内分配新的 UI 空间。
        ui.allocate_ui_at_rect(center_rect, |ui: &mut Ui| {
            let center_rect = ui.available_rect_before_wrap();
            // 将区域分成左右两半。
            let max_rect = left_half(&center_rect);
            // 在左半部分显示动作描述。
            ui.allocate_ui_at_rect(max_rect, |ui: &mut Ui| {
                desc(ui, "command_panel.toggle_open");
                desc(ui, "game.open_path_dialog");
                desc(ui, "game.open_bundle_dialog");
            });
            let max_rect = right_half(&center_rect);
            // 在右半部分显示对应的快捷键。
            ui.allocate_ui_at_rect(max_rect, |ui: &mut Ui| {
                keys(ui, "command_panel.toggle_open");
                keys(ui, "game.open_path_dialog");
                keys(ui, "game.open_bundle_dialog");
            });
        });
    });
}

/// 一个辅助函数，返回输入矩形的左半部分。
fn left_half(rect: &Rect) -> Rect {
    Rect::from_min_size(rect.min, [rect.width() / 2. - 10., rect.height()].into())
}

/// 一个辅助函数，返回输入矩形的右半部分。
fn right_half(rect: &Rect) -> Rect {
    Rect::from_min_max(
        [rect.max.x - rect.width() / 2. + 10., rect.min.y].into(),
        rect.max,
    )
}

/// `WindowUpdateControlPlugin` 插件用于控制窗口的更新行为。
/// 主要目的是在非播放状态下降低 CPU 使用率，只在需要时重绘窗口。
pub struct WindowUpdateControlPlugin;

impl Plugin for WindowUpdateControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, change_render_type) // 在启动时设置渲染模式。
            .add_systems(
                PostUpdate,
                // 仅当 `GameTime` 资源发生变化时（例如，在播放时），才运行 `update_type_changing` 系统。
                update_type_changing.run_if(resource_changed::<GameTime>),
            );
    }
}

/// `change_render_type` 系统在应用启动时运行，将窗口的 `PresentMode` 设置为 `AutoNoVsync`。
/// 这意味着窗口不会强制垂直同步，只在有输入或通过 `RequestRedraw` 请求时才会重绘。
fn change_render_type(mut window: Query<&mut Window, With<PrimaryWindow>>) -> Result<()> {
    window
        .single_mut()
        .map(|mut a| a.present_mode = PresentMode::AutoNoVsync)?;
    Ok(())
}

/// `update_type_changing` 系统在 `GameTime` 改变时运行，发送一个 `RequestRedraw` 事件。
/// 这确保了在谱面播放期间，窗口能够持续刷新以显示动画。
fn update_type_changing(mut event: EventWriter<RequestRedraw>) {
    event.send(RequestRedraw);
}

/// `MainUIPlugin` 是设置编辑器主 UI 的核心插件。
pub struct MainUIPlugin;

impl Plugin for MainUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_icons, spawn_cam)) // 启动时加载图标并生成相机。
            .add_systems(Update, (editor_main, input_state_update, setup_font));
        // 每帧更新时运行主 UI 逻辑。
    }
}
/// `spawn_cam` 系统在启动时生成一个 2D 相机，这是 Bevy 渲染 2D 场景所必需的。
fn spawn_cam(mut commands: Commands) {
    commands.spawn(Camera2d);
}

/// `setup_font` 系统用于为 egui 设置自定义字体。
/// 它只在 `EguiContext` 组件被添加时运行一次。
fn setup_font(mut context: Query<&mut EguiContext, Added<EguiContext>>) {
    use egui::{FontData, FontDefinitions, FontFamily};
    // 遍历所有新添加的 `EguiContext`。
    context.iter_mut().for_each(|mut c| {
        debug!("Setting up fonts for egui");
        let mut fonts = FontDefinitions::default();
        // 插入自定义的“思源黑体”字体数据。
        fonts.font_data.insert(
            "SourceHanSansSC".to_owned(),
            FontData::from_static(include_bytes!("../assets/SourceHanSansSC.otf")).into(),
        ); // .ttf and .otf supported
        fonts
            .families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "SourceHanSansSC".to_owned());
        // 将配置好的字体定义应用到 egui 上下文中。
        c.get_mut().set_fonts(fonts);
        debug!("Fonts set up successfully");
    });
}

/// `editor_main` 是每帧驱动整个编辑器 UI 渲染的主系统。
/// 它直接操作 `World`，因此可以访问任何资源和组件。
fn editor_main(world: &mut World) -> Result<()> {
    // 查询 `EguiContext`，这是与 egui 交互的入口点。
    let mut egui_context = world.query_filtered::<&mut EguiContext, ()>();
    let mut binding = egui_context.single_mut(world)?;
    // 克隆 egui 上下文，以便在多个 UI 函数之间传递。
    let ctx = binding.get_mut().clone();

    // 修改全局 egui 样式，设置边框宽度和窗口圆角。
    ctx.all_styles_mut(|style| {
        style.visuals.widgets.noninteractive.bg_stroke.width = 0.5;
        style.visuals.window_corner_radius = 0.0.into();
    });

    // 按顺序绘制顶部栏、主区域和底部状态栏。
    top_bar_ui(&ctx, world);
    main_ui(&ctx, world);
    bottom_ui(&ctx, world);
    Ok(())
}

/// `input_state_update` 系统用于更新 `EditorState` 中的 `is_editing_text` 字段。
fn input_state_update(
    mut editor_state: ResMut<EditorState>,
    mut window: Query<&mut EguiContext>,
) -> Result<()> {
    // 检查 egui 的输出，判断鼠标下方是否有可变的文本输入框。
    editor_state.is_editing_text = window
        .single_mut()?
        .get_mut()
        .output(|out| out.mutable_text_under_cursor);
    Ok(())
}

/// `top_bar_ui` 函数负责绘制编辑器顶部的菜单栏。
fn top_bar_ui(ctx: &egui::Context, world: &mut World) {
    // 获取 logo 的图像 ID。
    let r = world
        .resource::<EguiUserTextures>()
        .image_id(&world.resource::<Icons>().rizlium_colored_64x)
        .unwrap();
    // 创建一个固定高度的顶部面板。
    egui::TopBottomPanel::top("menu")
        .exact_height(35.0)
        .show(ctx, |ui| {
            // 运行命令面板的 widget 系统（如果面板是打开的）。
            widget(world, ui, command_panel);
            // 水平居中布局。
            ui.horizontal_centered(|ui| {
                // 显示 logo 图像。
                ui.add(
                    egui::Image::new((r, egui::Vec2::splat(23.0))), // .corner_radius(5),
                );
                // 使用 `resource_scope` 安全地访问 `MenuSystem` 资源并显示菜单。
                world.resource_scope(|world: &mut World, mut menu_system: Mut<MenuSystem>| {
                    // 应用自定义的主题。
                    ui.style_mut().visuals = menu_bar_theme();
                    // 显示主菜单。
                    menu_system.show_menu(ui, world, &MainMenuContext);
                });
                // 使用从右到左的布局来将 FPS 计数器推到最右边。
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    world.resource_scope(|world: &mut World, mut menu_system: Mut<MenuSystem>| {
                        // 应用自定义的主题。
                        ui.style_mut().visuals = menu_bar_theme();
                        // 显示右侧菜单。
                        menu_system.show_menu(ui, world, &RightMenuContext);
                    });
                });
            });
        });
}

/// `bottom_ui` 函数负责绘制编辑器底部的状态栏。
fn bottom_ui(ctx: &egui::Context, world: &mut World) {
    // TODO: 将状态栏逻辑迁移到独立的扩展中。
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            // 检查当前是否已加载谱面。
            if world
                .run_system_once(ProjectState::has_chart_system())
                .is_ok_and(|ok| ok)
            {
                // 如果已加载，显示谱面的基本信息。
                let chart = world.resource::<ProjectState>();
                ui.label("Ready");
                ui.separator();
                ui.label(format!("{} segments", chart.segment_count()));
                ui.separator();
                ui.label(format!("{} notes", chart.note_count()));
            } else {
                // 如果未加载，显示提示信息。
                ui.label("No chart loaded");
            }
        });
    });
}

/// `main_ui` 函数负责绘制编辑器的主工作区，主要是可停靠的窗口区域。
fn main_ui(ctx: &egui::Context, world: &mut World) {
    // 安全地访问 `TabRegistry` 和 `RizliumDockState` 资源。
    world.resource_scope(|world: &mut World, mut registry: Mut<'_, TabRegistry>| {
        world.resource_scope(
            |world: &mut World, mut state: Mut<'_, Persistent<RizliumDockState>>| {
                // 检查停靠区是否为空。
                if state.0.main_surface().is_empty() {
                    // 如果为空，显示一个中央面板作为欢迎界面。
                    egui::CentralPanel::default()
                        .frame(
                            // 设置深灰色背景。
                            Frame::central_panel(ctx.style().as_ref())
                                .fill(Color32::from_rgb(31, 31, 31)),
                        )
                        .show(ctx, |ui| {
                            // 运行 `ui_when_no_dock` widget 系统来绘制欢迎内容。
                            widget(world, ui, ui_when_no_dock);
                        });
                }
                // 显示 `DockArea`，这是所有可停靠窗口的容器。
                DockArea::new(&mut state.0).style(tab_theme(ctx)).show(
                    ctx,
                    // `HeTabViewer` 是 `egui-dock` 需要的 trait 实现，用于渲染每个标签页的内容。
                    &mut HeTabViewer {
                        registry: &mut registry,
                        world,
                    },
                );
                // 更新 `FocusedTab` 资源，以反映当前哪个标签页是激活的。
                world.resource_mut::<FocusedTab>().0 =
                    state.0.find_active_focused().unzip().1.cloned();
                // TODO: 将此逻辑移至更合适的文件。
            },
        );
    });
}
