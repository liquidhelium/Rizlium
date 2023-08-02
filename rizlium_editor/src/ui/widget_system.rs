use std::collections::HashMap;

use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use egui::{Id, Ui};
pub trait WidgetSystem: SystemParam {
    fn system(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, id: WidgetId);
}

pub fn widget<'ui, S: 'static + WidgetSystem>(world: &mut World, ui: &'ui mut Ui, id: WidgetId) {
    // We need to cache `SystemState` to allow for a system's locally tracked state
    if !world.contains_resource::<StateInstances<S>>() {
        // Note, this message should only appear once! If you see it twice in the logs, the function
        // may have been called recursively, and will panic.
        debug!("Init system state {}", std::any::type_name::<S>());
        world.insert_resource(StateInstances::<S> {
            instances: HashMap::new(),
        });
    }
    world.resource_scope(|world, mut states: Mut<StateInstances<S>>| {
        if !states.instances.contains_key(&id) {
            debug!(
                "Registering system state for widget {id:?} of type {}",
                std::any::type_name::<S>()
            );
            states.instances.insert(id, SystemState::new(world));
        }
        let cached_state = states.instances.get_mut(&id).unwrap();
        S::system(world, cached_state, ui, id);
        cached_state.apply(world);
    });
}

/// A UI widget may have multiple instances. We need to ensure the local state of these instances is
/// not shared. This hashmap allows us to dynamically store instance states.
#[derive(Default, Resource)]
struct StateInstances<T: WidgetSystem + 'static> {
    instances: HashMap<WidgetId, SystemState<T>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub Id);
impl WidgetId {
    pub fn new(name: impl Into<Id>) -> Self {
        WidgetId(name.into())
    }
    pub fn with(&self, name: &str) -> WidgetId {
        Self::new(self.0.with(name))
    }
}
