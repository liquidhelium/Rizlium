use bevy::{prelude::*, core_pipeline::clear_color::ClearColorConfig};

use crate::{chart::GameChart, time::GameTime, GameCamera, colorrgba_to_color};

pub struct BackgroundThemePlugin;

impl Plugin for BackgroundThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, change_bg);
    }
}

fn change_bg(chart: Res<GameChart>, time: Res<GameTime>, mut cam: Query<&mut Camera2d, With<GameCamera>>) {
    let theme = chart.theme_at(**time).unwrap();
    cam.single_mut().clear_color = ClearColorConfig::Custom(colorrgba_to_color(theme.this.color.background));
}