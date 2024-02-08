

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::EguiUserTextures;
use egui::{Button, Layout, Ui};

use rizlium_render::{GameTime, GameView, TimeManager};

use crate::{
    EditorCommands, TabProvider,
};

#[derive(SystemParam)]
pub struct GameViewTab<'w, 's> {
    gameview: Res<'w, GameView>,
    textures: Res<'w, EguiUserTextures>,
    time: Res<'w, TimeManager>,
    game_time: Res<'w, GameTime>,
    commands: EditorCommands<'s>,
}

impl<'w, 's> TabProvider for GameViewTab<'w, 's> {
    fn ui(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut Ui,
        _has_focus: bool,
    ) {
        let GameViewTab::<'_, '_> {
            gameview,
            textures,
            time,
            game_time,
            mut commands
        } = state.get(world);
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
        // video_control(ui, &mut false, 0.0..=100.0, &mut 50.);
        ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
            ui.allocate_ui_with_layout([90., 30.].into(), Layout::left_to_right(egui::Align::Center), |ui| {
                use rizlium_render::TimeControlEvent::*;
                if ui.add(Button::new("⏪").frame(false).min_size([30.;2].into())).clicked() {
                    commands.time_control(Advance(-1.));
                }
                let pause_play_icon = if time.paused() {
                    "▶"
                } else {
                    "⏸"
                };
                if ui.add(Button::new(pause_play_icon).frame(false).min_size([30.;2].into())).clicked() {
                    commands.time_control(Toggle);
                }
                if ui.add(Button::new("⏩").frame(false).min_size([30.;2].into())).clicked() {
                    commands.time_control(Advance(1.));
                }
            });
            keep_ratio(ui, 16. / 9., |ui, size| {
                let a: egui::TextureId = img;
                ui.centered_and_justified(|ui| ui.image(img, size));
            });
        });
    }
    fn name() -> String {
        "Game view".into()
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
