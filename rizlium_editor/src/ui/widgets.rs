mod dock_buttons;
mod recent_file_buttons;
pub use dock_buttons::dock_button;
pub use recent_file_buttons::recent_file_buttons;

use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use egui::Ui;
use std::fmt::Debug;
use strum::IntoEnumIterator;
pub trait WidgetSystem: SystemParam + 'static {
    type Extra<'a>;
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut Ui,
        _extra: Self::Extra<'_>,
    );
}

pub fn widget<W>(world: &mut World, ui: &mut Ui)
where
    W: WidgetSystem<Extra<'static> = ()>,
{
    widget_with::<W>(world, ui, ());
}
pub fn widget_with<W: WidgetSystem>(world: &mut World, ui: &mut Ui, extra: W::Extra<'_>) {
    if !world.contains_resource::<CachedWidgetState<W>>() {
        let value = CachedWidgetState(SystemState::<W>::new(world));
        world.insert_resource(value);
    }
    world.resource_scope(
        |world: &mut World, mut cache: Mut<'_, CachedWidgetState<W>>| {
            W::system(world, &mut cache.0, ui, extra);
            cache.0.apply(world);
        },
    );
}

#[derive(Resource)]
struct CachedWidgetState<W: SystemParam + 'static>(SystemState<W>);

pub fn enum_selector<T: IntoEnumIterator + Eq + Debug>(value: &mut T, ui: &mut Ui) {
    ui.menu_button(format!("{value:?}"), |ui| {
        for variant in T::iter() {
            let text = format!("{variant:?}");
            if ui.selectable_value(value, variant, text).changed() {
                ui.close_menu();
            };
        }
    });
}
