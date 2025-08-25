use bevy::prelude::*;
use std::ops::Deref;
#[derive(Resource, Reflect, Default)]
#[reflect(Resource, Default)]
pub struct GameTime(pub f32);
impl Deref for GameTime {
    type Target = f32;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct TimeAndAudioPlugin;
impl Plugin for TimeAndAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameTime>();
    }
}