use bevy::{prelude::*, ecs::system::SystemParam};
use rizlium_render::LoadChartEvent;

use crate::WidgetSystem;

#[derive(SystemParam)]
pub struct FileMenu<'w,'s> {
    events: EventWriter<'w ,LoadChartEvent>,
    stored_text: Local<'s,String>
}

impl WidgetSystem for FileMenu<'_,'_> {
    fn system(world: &mut World, state: &mut bevy::ecs::system::SystemState<Self>, ui: &mut egui::Ui, _id: super::widget_system::WidgetId) {
        let FileMenu::<'_,'_>{
            mut events,
            mut stored_text
        } = state.get_mut(world);
        ui.text_edit_singleline(&mut *stored_text);
        if ui.button("Submit").clicked() {
            events.send(LoadChartEvent(stored_text.clone()));
        }
        if ui.button("Submit_fast").clicked() {
            events.send(LoadChartEvent("/home/helium/code/rizlium/rizlium_render/assets/1.zip".to_owned()));
        }
    }
}