use std::marker::PhantomData;

use bevy::{prelude::*, render::camera::ClearColorConfig};

use crate::{colorrgba_to_color, default_ph, time_and_audio::GameTime, ChartProvider, GameCamera};

pub struct BackgroundThemePlugin<P: ChartProvider>(PhantomData<P>);

default_ph!(BackgroundThemePlugin<P>);

impl<P: ChartProvider> Plugin for BackgroundThemePlugin<P> {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            change_bg::<P>.run_if(
                P::has_chart_system().and(resource_changed::<P>.or(resource_changed::<GameTime>)),
            ),
        );
    }
}

fn change_bg<P: ChartProvider>(
    chart: Res<P>,
    time: Res<GameTime>,
    mut cam: Query<&mut Camera, With<GameCamera>>,
) {
    let theme = chart.chart().theme_at(**time).unwrap();
    if let Ok(mut camera) = cam.single_mut() {
        camera.clear_color =
            ClearColorConfig::Custom(colorrgba_to_color(theme.this.color.background));
    }
}
