use bevy::prelude::*;
use bevy_egui::EguiUserTextures;
use egui::Ui;
use rizlium_render::{GameTime, GameView, LoadChartEvent, TimeManager};

use crate::{
    extensions::MenuExt, hotkeys::{Hotkey, HotkeysExt}, menu::{self, Custom}, open_dialog, tab_system::TabRegistrationExt, widgets::{widget, RecentButtons}, ActionsExt, EditorCommands, PendingDialog
};
pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        use time_systems::*;
        use KeyCode::*;
        app.register_action("game.load_chart", "Load chart file", load_chart)
            .register_action(
                "game.open_dialog",
                "Open a dialog to pick chart file and load it",
                open_dialog_and_load_chart,
            )
            .register_action("game.time.advance", "Advance game time", advance_time)
            .register_action("game.time.rewind", "Rewind game time", rewind_time)
            .register_action(
                "game.time.toggle_pause",
                "Pause or resume game",
                toggle_pause,
            )
            .register_hotkey("game.open_dialog", [Hotkey::new_global([ControlLeft, O])])
            .register_hotkey("game.time.advance", [Hotkey::new_global([Right])])
            .register_hotkey("game.time.rewind", [Hotkey::new_global([Left])])
            .register_hotkey("game.time.toggle_pause", [Hotkey::new_global([Space])])
            .menu_context(|mut ctx| {
                ctx.with_sub_menu("file", "File".into(), 0, |mut ctx| {
                    ctx.add(
                        "open_chart",
                        "Open".into(),
                        menu::Button::new("game.open_dialog".into()),
                        0,
                    );
                    ctx.with_category("recent_files", "Recent Files".into(), 1, |mut ctx| {
                        ctx.add(
                            "recent_files_inner",
                            "_".into(),
                            Custom(Box::new(
                                |ui, world, _| widget::<RecentButtons>(world, ui),
                            )),
                            0,
                        );
                    })
                });
            })
            .register_tab("game.view".into(), "Game view", game_view_tab, || true);
    }
}

fn load_chart(
    path: In<String>,
    mut load: EventWriter<LoadChartEvent>,
    _to_recent_file: (), /* todo */
) {
    load.send(LoadChartEvent(path.0));
}

fn open_dialog_and_load_chart(mut dialog: ResMut<PendingDialog>) {
    open_dialog(&mut dialog)
}

mod time_systems {
    const SINGLE_TIME: f32 = 1.0;
    use rizlium_render::TimeControlEvent::*;

    use crate::EditorCommands;
    pub fn advance_time(mut commands: EditorCommands) {
        commands.time_control(Advance(SINGLE_TIME))
    }
    pub fn rewind_time(mut commands: EditorCommands) {
        commands.time_control(Advance(-SINGLE_TIME))
    }
    pub fn toggle_pause(mut commands: EditorCommands) {
        commands.time_control(Toggle)
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
    In(ui): In<&mut Ui>,
    gameview: Res<GameView>,
    textures: Res<EguiUserTextures>,
    time: Res<TimeManager>,
    game_time: Res<GameTime>,
    mut commands: EditorCommands,
) {
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
            ui.allocate_ui_with_layout(
                [90., 30.].into(),
                Layout::left_to_right(egui::Align::Center),
                |ui| {
                    use rizlium_render::TimeControlEvent::*;
                    if ui
                        .add(Button::new("⏪").frame(false).min_size([30.; 2].into()))
                        .clicked()
                    {
                        commands.time_control(Advance(-1.));
                    }
                    let pause_play_icon = if time.paused() { "▶" } else { "⏸" };
                    if ui
                        .add(
                            Button::new(pause_play_icon)
                                .frame(false)
                                .min_size([30.; 2].into()),
                        )
                        .clicked()
                    {
                        commands.time_control(Toggle);
                    }
                    if ui
                        .add(Button::new("⏩").frame(false).min_size([30.; 2].into()))
                        .clicked()
                    {
                        commands.time_control(Advance(1.));
                    }
                },
            );
            keep_ratio(ui, 16. / 9., |ui, size| {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::new((img, size)).fit_to_exact_size(size))
                });
            });
        });
}
