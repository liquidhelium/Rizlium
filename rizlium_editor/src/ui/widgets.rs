mod dock_buttons;
mod preset;
pub use dock_buttons::DockButtons;
pub use preset::{PresetButtons, LayoutPresetEdit};

use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use egui::Ui;
pub trait WidgetSystem: SystemParam + 'static {
    type Extra<'a>;
    fn system<'a>(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, _extra: Self::Extra<'a>);
}

pub fn widget<W>(world: &mut World, ui: &mut Ui)
where
    W: WidgetSystem<Extra<'static> = ()>,
{
    widget_with::<W>(world, ui, ());
}
pub fn widget_with<'a,W: WidgetSystem>(world: &mut World, ui: &mut Ui, extra: W::Extra<'a>) {
    if !world.contains_resource::<CachedWidgetState<W>>() {
        let value = CachedWidgetState(SystemState::<W>::new(world));
        world.insert_resource(value);
    }
    world.resource_scope(
        |world: &mut World, mut cache: Mut<'_, CachedWidgetState<W>>| {
            W::system(world, &mut cache.0, ui, extra);
        },
    );
}

#[derive(Resource)]
struct CachedWidgetState<W: SystemParam + 'static>(SystemState<W>);