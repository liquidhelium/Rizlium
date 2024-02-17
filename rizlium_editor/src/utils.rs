use bevy::ecs::system::{Res, SystemParam};
use rizlium_render::{GameChartCache, GameTime};

pub mod dot_path;

#[derive(SystemParam)]
pub struct WorldToGame<'w> {
    pub cache: Option<Res<'w, GameChartCache>>,
    pub time: Option<Res<'w, GameTime>>,
}

impl WorldToGame<'_> {
    pub fn time_at_y(&self, world_y: f32, canvas: usize) -> Option<f32> {
        self.cache.as_deref()?
            .canvas_y_to_time(canvas, world_y + self.cache.as_deref()?.canvas_y_at(canvas, **self.time.as_deref()?)?)
    }
    pub fn avalible(&self) -> bool {
        self.cache.is_some() && self.time.is_some()
    }
}
