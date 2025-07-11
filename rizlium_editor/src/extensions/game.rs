use bevy::{
    input::mouse::MouseWheel,
    prelude::*,
    render::render_resource::{
        Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    }, transform::commands,
};
use bevy_egui::{EguiContexts, EguiUserTextures};
use egui::Ui;
use rizlium_render::{notes::NoteTexture, GameChart, GameTime, GameView, TimeControlEvent, TimeManager};
use rust_i18n::t;

use crate::{open_dialog, save_chart, widgets::recent_file_buttons, LoadChartEvent, PendingDialog};
use helium_framework::{
    menu::{self, Custom, MenuExt},
    prelude::*,
    widgets::widget,
};
pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        use time_systems::*;
        use KeyCode::*;
        app.reflect_system("game.load_chart", "Load chart file", load_chart)
            .reflect_system("game.save_chart", "Save current chart to file", save_chart)
            .reflect_system(
                "game.open_dialog",
                "Open a dialog to pick chart file and load it",
                open_dialog_and_load_chart,
            )
            .reflect_system("game.time.advance", "Advance game time", advance_time)
            .reflect_system("game.time.rewind", "Rewind game time", rewind_time)
            .reflect_system(
                "game.time.toggle_pause",
                "Pause or resume game",
                toggle_pause,
            )
            .reflect_system("game.time.control", "Control game time", time_control)
            .reflect_system(
                "game.time.enable_scroll_time",
                "Enable scrolling to change time",
                toggle_enable_scroll_time,
            )
            .register_hotkey(
                "game.open_dialog",
                [Hotkey::new_global([ControlLeft, KeyO])],
            )
            .register_hotkey(
                "game.save_chart",
                [Hotkey::new(
                    [ControlLeft, KeyS],
                    resource_exists::<GameChart>,
                )],
            )
            .register_hotkey("game.time.advance", [Hotkey::new_global([ArrowRight])])
            .register_hotkey("game.time.rewind", [Hotkey::new_global([ArrowLeft])])
            .register_hotkey("game.time.toggle_pause", [Hotkey::new_global([Space])])
            .register_hotkey(
                "game.time.enable_scroll_time",
                [Hotkey::new_advanced(
                    [ControlLeft],
                    tab_focused("game.view"),
                    TriggerType::PressAndRelease,
                )],
            )
            .menu_context(|mut ctx| {
                ctx.with_sub_menu("file", t!("file.tab"), 0, |mut ctx| {
                    ctx.add(
                        "open_chart",
                        t!("action.open_chart"),
                        menu::Button::new("game.open_dialog"),
                        0,
                    );
                    ctx.add(
                        "save_chart",
                        t!("action.save_chart"),
                        menu::Button::new_conditioned(
                            "game.save_chart",
                            resource_exists::<GameChart>,
                        ),
                        1,
                    );
                    ctx.with_category("recent_files", t!("file.recent_files"), 2, |mut ctx| {
                        ctx.add(
                            "recent_files_inner",
                            "_".into(),
                            Custom(Box::new(|ui, world, _| {
                                widget(world, ui, recent_file_buttons)
                            })),
                            0,
                        );
                    })
                });
            })
            .register_tab("game.view", t!("game.view.tab"), game_view_tab, || true);
        // bevy systems
        app.add_systems(
            Startup,
            (setup_game_view.after(bevy_egui::EguiStartupSet::InitContexts), load_textures),
        )
        .add_systems(Update, scroll_time)
        .init_resource::<ScrollTimeState>();
    }
}

fn load_textures(server: Res<AssetServer>,mut commands: Commands) {
    commands.insert_resource(NoteTexture {
        note_frame: server.load("note_textures/note_frame.png"),
        note_bg: server.load("note_textures/note_bg.png"),
        hold_body: server.load("note_textures/hold_body.png"),
        hold_cap: server.load("note_textures/hold_cap.png"),
        drag: server.load("note_textures/drag.png"),
    });
}

fn load_chart(
    path: In<String>,
    mut load: EventWriter<LoadChartEvent>,
    _to_recent_file: (), /* todo */
) {
    load.write(LoadChartEvent(path.0));
}

fn open_dialog_and_load_chart(mut dialog: ResMut<PendingDialog>) {
    open_dialog(&mut dialog)
}

#[derive(Resource, Default)]
struct ScrollTimeState(bool);

fn toggle_enable_scroll_time(In(trigger): In<RuntimeTrigger>, mut state: ResMut<ScrollTimeState>) {
    state.0 = trigger.is_pressed();
}

fn scroll_time(
    state: Res<ScrollTimeState>,
    mut wheel: EventReader<MouseWheel>,
    mut time: EventWriter<TimeControlEvent>,
) {
    if state.0 {
        for i in wheel.read() {
            time.write(TimeControlEvent::Advance(i.y * 0.01));
        }
    }
}

mod time_systems {
    const SINGLE_TIME: f32 = 1.0;
    use bevy::ecs::{event::EventWriter, system::In};
    use rizlium_render::TimeControlEvent::{self, *};
    pub fn advance_time(mut ev: EventWriter<TimeControlEvent>) {
        ev.write(Advance(SINGLE_TIME));
    }
    pub fn rewind_time(mut ev: EventWriter<TimeControlEvent>) {
        ev.write(Advance(-SINGLE_TIME));
    }
    pub fn toggle_pause(mut ev: EventWriter<TimeControlEvent>) {
        ev.write(Toggle);
    }
    pub fn time_control(In(event): In<TimeControlEvent>, mut ev: EventWriter<TimeControlEvent>) {
        ev.write(event);
    }
}

fn keep_ratio(ui: &mut Ui, ratio: f32, mut add_fn: impl FnMut(&mut Ui, egui::Vec2)) {
    assert_ne!(ratio, 0.);
    let current_size = ui.available_size();
    let mut new_size = egui::Vec2::default();
    if current_size.x < current_size.y / ratio {
        new_size.x = current_size.x;
        new_size.y = current_size.x * ratio;
    } else {
        new_size.x = current_size.y / ratio;
        new_size.y = current_size.y;
    }
    add_fn(ui, new_size);
}

pub fn game_view_tab(
    InMut(mut ui): InMut<Ui>,
    gameview: Res<GameView>,
    textures: Res<EguiUserTextures>,
    time: Res<TimeManager>,
    game_time: Res<GameTime>,
    mut ev: EventWriter<TimeControlEvent>,
) {
    let ui = &mut ui;
    let img = textures
        .image_id(&gameview.0)
        .expect("no gameview image found!");
    egui::TopBottomPanel::top("gameview top bar").show_inside(ui, |ui| {
        ui.horizontal_top(|ui| {
            ui.label(format!("Real: {:.2}", time.current()));
            ui.separator();
            ui.label(format!("Game: {:.2}", **game_time));
            ui.separator();
            ui.menu_button("title", |ui| {
                ui.label("text");
            });
        });
    });
    use egui::*;
    // video_control(ui, &mut false, 0.0..=100.0, &mut 50.);
    ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
        // here used to be control buttons, but now we just have the game
        keep_ratio(ui, 16. / 9., |ui, size| {
            ui.centered_and_justified(|ui| {
                ui.add(egui::Image::new((img, size)).fit_to_exact_size(size))
            });
        });
    });
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
