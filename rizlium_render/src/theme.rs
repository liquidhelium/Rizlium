use bevy::{core_pipeline::clear_color::ClearColorConfig, prelude::*};

use crate::{chart::GameChart, colorrgba_to_color, time_and_audio::GameTime, GameCamera};

pub struct BackgroundThemePlugin;

impl Plugin for BackgroundThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, change_bg.run_if(resource_exists::<GameChart>()
        .and_then(resource_changed::<GameChart>().or_else(resource_changed::<GameTime>()))
));
    }
}

fn change_bg(
    chart: Res<GameChart>,
    time: Res<GameTime>,
    mut cam: Query<&mut Camera2d, With<GameCamera>>,
) {
    let theme = chart.theme_at(**time).unwrap();
    cam.single_mut().clear_color =
        ClearColorConfig::Custom(colorrgba_to_color(theme.this.color.background));
}
