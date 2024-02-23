use bevy::ecs::{schedule::{BoxedCondition, Condition}, system::{IntoSystem, Res, SystemParam, System,}};
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

pub fn new_condition<M>(condition: impl Condition<M>) -> BoxedCondition {
    let condition_system = IntoSystem::into_system(condition);
    assert!(
        condition_system.is_send(),
        "Condition `{}` accesses `NonSend` resources. This is not currently supported.",
        condition_system.name()
    );

    Box::new(condition_system)
}