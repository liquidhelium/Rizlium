use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiUserTextures};
use egui::Ui;
use rizlium_render::{TimeManager, GameView};

use crate::EditorState;
pub fn game_view(ui: &mut Ui,world: &mut World, _: &mut EditorState) {
    let gameview = world.resource::<GameView>();
    let ctx = world.resource::<EguiUserTextures>();
    let img = ctx
        .image_id(&gameview.0)
        .expect("no gameview image found!");
    
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {

        let button = &ui.button("暂停");
        if button.clicked() {
            world.resource_mut::<TimeManager>().toggle_paused();
        }
        
        keep_ratio(ui, 16./9., |ui, size| {
            ui.centered_and_justified(|ui| ui.image(img, size));
            
        })
        
    });
}
fn keep_ratio(ui: &mut Ui, ratio: f32,mut add_fn: impl FnMut(&mut Ui, egui::Vec2)) {
    assert_ne!(ratio, 0.);
    let current_size = ui.available_size();
    let mut new_size = egui::Vec2::default();
    if current_size.x < current_size.y/ ratio {
        new_size.x = current_size.x;
        new_size.y = current_size.x*ratio;
    }
    else {
        new_size.x = current_size.y/ratio;
        new_size.y = current_size.y;
    }
    add_fn(ui, new_size);

}