use bevy::{prelude::*, render::camera::ClearColorConfig};

use crate::{chart::GameChart, colorrgba_to_color, time_and_audio::GameTime, GameCamera};

pub struct BackgroundThemePlugin;

impl Plugin for BackgroundThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            change_bg.run_if(
                resource_exists::<GameChart>
                    .and(resource_changed::<GameChart>.or(resource_changed::<GameTime>)),
            ),
        );
    }
}

fn change_bg(
    chart: Res<GameChart>,
    time: Res<GameTime>,
    mut cam: Query<&mut Camera, With<GameCamera>>,
) {
    let theme = chart.theme_at(**time).unwrap();
    if let Ok(mut camera) = cam.single_mut() {
        camera.clear_color =
            ClearColorConfig::Custom(colorrgba_to_color(theme.this.color.background));
    }
}
