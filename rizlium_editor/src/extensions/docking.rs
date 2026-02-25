use crate::{
    settings_module::{SettingsModuleStruct, SettingsRegistrationExt},
    widgets::dock_button,
    MainMenuContext, RizTabPresets, RizliumDockStateMirror,
};
use bevy::{asset::uuid::Uuid, prelude::*};
use bevy_persistent::Persistent;
use egui::{Sense, TextEdit, Ui, Widget};
use helium_framework::{
    menu_system::MenuRegistration,
    prelude::{ActionsExt, TabRegistry, ToastsStorage},
    utils::identifier::Identifier,
    widgets::widget,
};
#[allow(unused_imports)]
use crate::t;
pub struct Docking;

impl Plugin for Docking {
    fn build(&self, app: &mut App) {
        app.reflect_system(
            "docking.button",
            "A docking Button",
            |(InMut(ui), InRef(_)): (InMut<Ui>, InRef<MainMenuContext>), world: &mut World| {
                widget(world, ui, dock_button);
            },
        );

        app.reflect_system("docking.open_tab", "Open a certain tab", open_certain_tab);
        app.register_submenu::<MainMenuContext>("window", "Window");
        app.register_custom::<MainMenuContext>("window/button", "Docking", "docking.button");
        app.register_settings_module(
            "docking",
            SettingsModuleStruct::new(
                docking_ui_module,
                apply_docking_settings,
                t!("settings-docking"),
            ),
        );
    }
}
struct DockSettingState {
    current_editing_name: Option<Uuid>,
    selected_preset: Uuid,
    temp_presets: RizTabPresets,
}
type Storage = DockSettingState;
fn docking_ui_module(
    In((mut ui, mut state)): In<(Ui, Option<Storage>)>,
    presets: Res<Persistent<RizTabPresets>>,
    current: Res<RizliumDockStateMirror>,
) -> Option<Storage> {
    let current = current.0.as_ref()?;
    let mut changed = false;
    if state.is_none() {
        state = Some(DockSettingState {
            current_editing_name: None,
            selected_preset: Uuid::nil(),
            temp_presets: presets.clone(),
        });
    } else {
        changed = true;
    }
    let mut state = state.unwrap();
    let mut to_delete_index: Option<usize> = None;
    ui.heading("Docking settings");
    egui::ScrollArea::vertical().show(&mut ui, |ui| {
        let current_value = &mut state.selected_preset;
        for (index, (uuid, name, _preset)) in state.temp_presets.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if state.current_editing_name == Some(*uuid) {
                    let response = TextEdit::singleline(name)
                        .id("current_edit".into())
                        .desired_width(50.0)
                        .ui(ui)
                        .on_hover_text("Click outside to cancel");
                    if response.lost_focus() {
                        state.current_editing_name = None;
                        changed = true;
                        if name.is_empty() {
                            *name = "Preset".into();
                        }
                    }
                } else if ui
                    .add(egui::Label::new(name.as_str()).sense(Sense::click()))
                    .on_hover_text("Double click to edit")
                    .double_clicked()
                {
                    state.current_editing_name = Some(*uuid);
                    changed = true;
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Delete").clicked() {
                        to_delete_index = Some(index);
                        changed = true;
                    }
                    if ui.radio_value(current_value, *uuid, "").changed() {
                        changed = true;
                    }
                });
            });
        }
    });
    if ui.button("Add new preset").clicked() {
        let new_name = format!("Preset {}", state.temp_presets.len() + 1);
        let mut current = current.clone();
        current
            .main_surface_mut()
            .retain_tabs(|tab| tab != &mut Identifier::from("settings"));
        state.temp_presets.push((Uuid::new_v4(), new_name, current));
        changed = true;
    }
    if let Some(index) = to_delete_index {
        state.temp_presets.remove(index);
        changed = true;
    }
    if changed {
        Some(state)
    } else {
        None
    }
}
fn apply_docking_settings(
    In(storage): In<Storage>,
    mut current: ResMut<RizliumDockStateMirror>,
    mut presets: ResMut<Persistent<RizTabPresets>>,
    mut toast: ResMut<ToastsStorage>,
) {
    if let Some(preset) = storage
        .temp_presets
        .iter()
        .find(|(id, _, _)| *id == storage.selected_preset)
    {
        current.0 = Some(preset.2.clone());
    } else {
        warn!("Selected a non-existing docking preset");
    }
    if let Err(e) = presets.set(storage.temp_presets) {
        error!("Failed to save docking presets: {}", e);
        toast.error(t!("settings-docking-save-error"));
    } else {
        info!("Docking presets saved successfully");
    }
}

fn open_certain_tab(
    In(tab_id): In<Identifier>,
    mut state: ResMut<Persistent<crate::RizliumDockState>>,
    registry: Res<TabRegistry>,
) {
    let state = &mut state.0;
    if registry.get(&tab_id).is_some() {
        state.add_window(vec![tab_id]);
    } else {
        warn!("Tried to open a non-existing tab: {}", tab_id);
    }
}

fn close_certain_tab(
    In(tab_id): In<Identifier>,
    mut state: ResMut<Persistent<crate::RizliumDockState>>,
    _registry: Res<TabRegistry>,
) {
    let state = &mut state.0;
    if let Some(tab) = state.find_tab(&tab_id) {
        state.remove_tab(tab);
    } else {
        warn!("Tried to close a non-existing tab: {}", tab_id);
    }
}
