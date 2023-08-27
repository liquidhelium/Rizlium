use std::ops::RangeInclusive;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::EguiUserTextures;
use egui::{Button, Color32, Layout, RichText, Ui};
use rizlium_render::{GameTime, GameView, TimeManager};

use crate::TabProvider;

#[derive(SystemParam)]
pub struct GameViewTab<'w> {
    gameview: Res<'w, GameView>,
    textures: Res<'w, EguiUserTextures>,
    time: ResMut<'w, TimeManager>,
    game_time: Res<'w, GameTime>,
}

impl<'w> TabProvider for GameViewTab<'w> {
    fn system(world: &mut World, state: &mut bevy::ecs::system::SystemState<Self>, ui: &mut Ui) {
        let GameViewTab::<'_> {
            gameview,
            textures,
            mut time,
            game_time,
        } = state.get_mut(world);
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
        keep_ratio(ui, 16. / 9., |ui, size| {
            ui.centered_and_justified(|ui| ui.image(img, size));
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
