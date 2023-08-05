use std::ops::RangeInclusive;

use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_egui::EguiUserTextures;
use egui::{Button, Color32, Layout, RichText, Ui};
use rizlium_render::{GameView, TimeManager};

use crate::{TabProvider};

#[derive(SystemParam)]
pub struct GameViewTab<'w> {
    gameview: Res<'w, GameView>,
    textures: Res<'w, EguiUserTextures>,
    time: ResMut<'w, TimeManager>,
}

impl<'w> TabProvider for GameViewTab<'w> {
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut Ui,
    ) {
        let GameViewTab::<'_> {
            gameview,
            textures,
            mut time,
        } = state.get_mut(world);
        let img = textures
            .image_id(&gameview.0)
            .expect("no gameview image found!");
        egui::TopBottomPanel::top("gameview top bar").show_inside(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.label(format!("Real: {:.2}", time.current()));
                ui.separator();
                // ui.label(format!("Game: {:.2}", **world.resource::<GameTime>()));
                ui.separator();
                ui.menu_button("title", |ui| {
                    ui.label("text");
                });
            });
        });

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            let button = &ui.button("暂停");
            if button.clicked() {
                time.toggle_paused();
            }
            // video_control(ui, &mut false, 0.0..=100.0, &mut 50.);
            keep_ratio(ui, 16. / 9., |ui, size| {
                ui.centered_and_justified(|ui| ui.image(img, size));
            })
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

fn video_control(
    ui: &mut Ui,
    paused: &mut bool,
    time_range: RangeInclusive<f32>,
    current: &mut f32,
) {
    ui.with_layout(Layout::bottom_up(egui::Align::Center), |ui| {
        let pause_icon = if *paused { "▶" } else { "⏸" };
        ui.add(Button::new(RichText::from(pause_icon).size(18.)).frame(false));
        let (_seekbar_id, seekbar_rect) = ui.allocate_space([ui.available_width(), 20.].into());
        let seekbar_main_rect = seekbar_rect.shrink2([20., 8.].into());
        ui.painter()
            .rect_filled(seekbar_main_rect, 0., Color32::GRAY);
        let mut seekbar_current_rect = seekbar_rect.clone();
        *seekbar_current_rect.right_mut() = seekbar_main_rect.width()
            * egui::emath::inverse_lerp(time_range, *current).unwrap_or(0.)
            + seekbar_main_rect.left();
        ui.painter()
            .rect_filled(seekbar_main_rect, 0., Color32::BLUE);
    });
}
