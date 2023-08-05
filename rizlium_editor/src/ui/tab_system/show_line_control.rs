use bevy::{ecs::system::SystemParam, prelude::*, render::view::RenderLayers};
use rizlium_render::{GameCamera, GameChart, ShowLines};

use crate::TabProvider;

#[derive(SystemParam)]
pub struct ShowLineControl<'w, 's> {
    commands: Commands<'w, 's>,
    chart: Res<'w, GameChart>,
    game_cam: Query<'w, 's, Entity, With<GameCamera>>,
    current_show: ResMut<'w, ShowLines>,
}

impl TabProvider for ShowLineControl<'_, '_> {
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
    ) {
        let ShowLineControl::<'_, '_> {
            mut commands,
            chart,
            mut game_cam,
            mut current_show,
        } = state.get_mut(world);
        let mut value = current_show.0.is_some();
        // if current_show.is_changed() {
            if value {
                commands
                    .entity(game_cam.single_mut())
                    .insert(RenderLayers::layer(1));
            } else {
                commands
                    .entity(game_cam.single_mut())
                    .insert(RenderLayers::layer(0));
            }
        // }
        ui.checkbox(&mut value, "Show");
        if value {
            if !current_show.0.is_some() {
                current_show.0 = Some(0);
            }
        } else {
            current_show.0.take();
        }
        if let Some(current) = current_show.0.as_mut() {
            ui.scope(|ui| {
                ui.style_mut().spacing.slider_width = 500.;
                ui.add(egui::Slider::new(current, 0..=chart.lines.len()));
            });
            ui.text_edit_multiline(&mut format!("{:#?}", chart.lines.get(*current)));
        }
    }
    fn name() -> String {
        "Line inspector".into()
    }

    fn avaliable(world: &World) -> bool {
        world.contains_resource::<GameChart>()
    }
}
