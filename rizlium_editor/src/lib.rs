#![allow(clippy::too_many_arguments)]

use std::{path::PathBuf, time::Duration};

use bevy::{
    diagnostic::FrameCount,
    ecs::system::RunSystemOnce as _,
    prelude::*,
    window::{PresentMode, PrimaryWindow, RequestRedraw},
};
use bevy_egui::{EguiContext, EguiContexts, EguiGlobalSettings, EguiPrimaryContextPass, EguiUserTextures, PrimaryEguiContext};
use bevy_persistent::Persistent;
use egui::{Color32, Frame, Label, Layout, Rect, RichText, Style, Ui, UiBuilder, Widget};
use egui_dock::DockArea;
use helium_framework::{
    menu_system::MenuSystem,
    prelude::{FocusedTab, HeTabViewer, HotkeyRegistry, RSystemRegistry, TabRegistry},
    utils::identifier::Identifier,
    widgets::widget,
};
// use egui_tracing::EventCollector;
use rizlium_render::{ChartProvider as _, GameTime};
i18n!();

use rust_i18n::i18n;
pub use ui::*;
mod editor_actions;
pub mod extensions;
pub mod project;
pub mod settings_module;
pub mod time_and_audio;
pub mod utils;
pub use editor_actions::*;
pub use project::*;

use crate::{
    extensions::command_panel::command_panel,
    ui::{theme::{tab_theme, top_bar_theme}, widgets::shortcut_display},
};

#[derive(Debug)]
pub struct MainMenuContext;
mod ui;
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

macro_rules! icons_def {
    ($stru:ident, $func:ident, ($($path:ident),+)) => {
        #[derive(Resource)]
        pub struct $stru {
            $($path: Handle<Image>),+
        }


        fn load_icons(
            mut commands: Commands,
            asset_server: Res<AssetServer>,
            mut egui_context: EguiContexts,
        ) -> Result<()> {
            $(
                let $path = asset_server.load(concat!(stringify!($path), ".png"));
            )+
            commands.insert_resource($stru {
                $($path: $path.clone(),)+
            });
            $(
                egui_context.add_image($path);
            )+
            Ok(())
        }

    };
}

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

pub fn ui_when_no_dock(
    In(ui): In<&mut Ui>,
    recents: Res<Persistent<RecentFiles>>,
    mut events: EventWriter<LoadChartEvent>,
    egui_textures: Res<EguiUserTextures>,
    icons: Res<Icons>,
    hotkeys: Res<HotkeyRegistry>,
    actions: Res<RSystemRegistry>,
) {
    let desc = |ui: &mut Ui, id: &str| {
        if let Some(action) = actions.get(&Identifier::from(id)) {
            ui.with_layout(Layout::right_to_left(egui::Align::Min), |ui| {

                ui.label(RichText::new(action.description.clone()).weak())
            });
        }
    };
    let keys = |ui: &mut Ui, id: &str| {
        if let Some(hotkeys) = hotkeys.get(&Identifier::from(id)) {
            ui.with_layout(Layout::left_to_right(egui::Align::Min), |ui| {
                ui.set_style(Style::default());
                let max_index = hotkeys.len() -1;
                for (index,hotkey) in hotkeys.iter().enumerate() {
                    shortcut_display(&hotkey.key.iter().map(|&key| format!("{key:?}")).collect::<Vec<_>>(), ui);
                    if index != max_index {
                        ui.label(";");
                    }
                }
            });
        }
    };
    let id = egui_textures.image_id(&icons.rizlium_solid_darker).unwrap();
    let main_rect = ui.available_rect_before_wrap().shrink(50.);
    ui.allocate_new_ui(UiBuilder::new().max_rect(main_rect), |ui: &mut Ui| {
        ui.vertical_centered(|ui| {
            ui.add(egui::Image::new((id, egui::Vec2::splat(300.))));
            
        });
        let main_rect = ui.available_rect_before_wrap().shrink(50.);
        let center_rect = if main_rect.width() >= 500. {
            Rect::from_center_size(main_rect.center(), [500., main_rect.height()].into())
        } else {
            main_rect
        };
        ui.allocate_new_ui(UiBuilder::new().max_rect(center_rect), |ui: &mut Ui| {
            let center_rect = ui.available_rect_before_wrap();
            let max_rect = left_half(&center_rect);
            ui.allocate_new_ui(UiBuilder::new().max_rect(max_rect), |ui: &mut Ui| {
                desc(ui, "command_panel.toggle_open");
                desc(ui, "game.open_path_dialog");
                desc(ui, "game.open_bundle_dialog");
            });
            let max_rect = right_half(&center_rect);
            ui.allocate_new_ui(UiBuilder::new().max_rect(max_rect), |ui: &mut Ui| {
                keys(ui, "command_panel.toggle_open");
                keys(ui, "game.open_path_dialog");
                keys(ui, "game.open_bundle_dialog");
            });
        });
    });
}

fn left_half(rect: &Rect) -> Rect {
    Rect::from_min_size(rect.min, [rect.width() / 2. - 10., rect.height()].into())
}

fn right_half(rect: &Rect) -> Rect {
    Rect::from_min_max(
        [rect.max.x - rect.width() / 2. + 10., rect.min.y].into(),
        rect.max,
    )
}

fn do114514<const LEN: usize>() -> String {
    ["114514"; LEN].join("")
}

pub struct WindowUpdateControlPlugin;

impl Plugin for WindowUpdateControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, change_render_type).add_systems(
            PostUpdate,
            update_type_changing.run_if(resource_changed::<GameTime>),
        );
    }
}

fn change_render_type(mut window: Query<&mut Window, With<PrimaryWindow>>) -> Result<()> {
    window
        .single_mut()
        .map(|mut a| a.present_mode = PresentMode::AutoNoVsync)?;
    Ok(())
}

fn update_type_changing(mut event: EventWriter<RequestRedraw>) {
    event.write(RequestRedraw);
}

pub struct MainUIPlugin;

impl Plugin for MainUIPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (load_icons, spawn_cam))
            .add_systems(EguiPrimaryContextPass, (editor_main, input_state_update, setup_font));
    }
}
fn spawn_cam(mut commands: Commands, mut settings: ResMut<EguiGlobalSettings>) {
    settings.auto_create_primary_context = false;
    commands.spawn((Camera2d, PrimaryEguiContext));
}

fn setup_font(mut context: Query<&mut EguiContext, Added<EguiContext>>) {
    use egui::{FontData, FontDefinitions, FontFamily};
    context.iter_mut().for_each(|mut c| {
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

fn editor_main(world: &mut World) -> Result {
    let mut egui_context = world.query_filtered::<&mut EguiContext, With<PrimaryEguiContext>>();
    let mut binding = egui_context.single_mut(world)?;
    let ctx = &binding.get_mut().clone();

    ctx.all_styles_mut(|style| {
        style.visuals.widgets.noninteractive.bg_stroke.width = 0.5;
        style.visuals.window_corner_radius = 0.0.into();
    });

    top_bar_ui(ctx, world);
    main_ui(ctx, world);
    bottom_ui(ctx, world);
    Ok(())
}

fn input_state_update(
    mut editor_state: ResMut<EditorState>,
    mut window: Query<&mut EguiContext, With<PrimaryEguiContext>>,
) -> Result {
    editor_state.is_editing_text = window
        .single_mut()?
        .get_mut()
        .output(|out| out.mutable_text_under_cursor);
    Ok(())
}

fn top_bar_ui(ctx: &egui::Context, world: &mut World) {
    let r = world
        .resource::<EguiUserTextures>()
        .image_id(&world.resource::<Icons>().rizlium_colored_64x)
        .unwrap();
    egui::TopBottomPanel::top("menu")
        .exact_height(35.0)
        .show(ctx, |ui| {
            ui.horizontal_centered(|ui| {
                ui.add(
                    egui::Image::new((r, egui::Vec2::splat(23.0))), // .corner_radius(5),
                );
                world.resource_scope(|world: &mut World, mut menu_system: Mut<MenuSystem>| {
                    ui.style_mut().visuals = top_bar_theme();
                    menu_system.show_menu(ui, world, &MainMenuContext);
                });
                ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                    world.resource_scope(|_world, fps: Mut<'_, NowFps>| {
                        ui.label(format!("fps: {}", fps.0));
                    });
                });
            });
            widget(world, ui, command_panel);
        });
}

fn bottom_ui(ctx: &egui::Context, world: &mut World) {
    // todo: status into extension
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal_centered(|ui| {
            if world
                .run_system_once(ProjectState::has_chart_system())
                .is_ok_and(|ok| ok)
            {
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
}

fn main_ui(ctx: &egui::Context, world: &mut World) {
    world.resource_scope(|world: &mut World, mut registry: Mut<'_, TabRegistry>| {
        world.resource_scope(
            |world: &mut World, mut state: Mut<'_, Persistent<RizliumDockState>>| {
                if state.0.main_surface().is_empty() {
                    egui::CentralPanel::default()
                        .frame(
                            Frame::central_panel(ctx.style().as_ref())
                                .fill(Color32::from_rgb(31, 31, 31)),
                        )
                        .show(ctx, |ui| {
                            widget(world, ui, ui_when_no_dock);
                        });
                }
                DockArea::new(&mut state.0).style(tab_theme(ctx)).show(
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
}
