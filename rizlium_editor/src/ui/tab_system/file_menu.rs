use bevy::{ecs::system::SystemParam, prelude::*};
use rizlium_render::LoadChartEvent;

use crate::{hotkeys::NoAction, TabProvider};

#[derive(SystemParam)]
pub struct FileMenu<'w, 's> {
    events: EventWriter<'w, LoadChartEvent>,
    stored_text: Local<'s, String>,
}

impl TabProvider for FileMenu<'_, '_> {
    type Hotkey = NoAction;
    fn ui(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        has_focus: bool,
    ) {
        let FileMenu::<'_, '_> {
            mut events,
            mut stored_text,
        } = state.get_mut(world);
        ui.text_edit_singleline(&mut *stored_text);
        if ui.button("Submit").clicked() {
            events.send(LoadChartEvent(stored_text.clone()));
        }
        if ui.button("Submit_fast").clicked() {
            events.send(LoadChartEvent(
                "/home/helium/code/rizlium/rizlium_render/assets/1.zip".to_owned(),
            ));
        }
    }
    fn name() -> String {
        "Load files (Debug)".into()
    }
}
