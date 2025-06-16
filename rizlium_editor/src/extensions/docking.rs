
use bevy::{asset::uuid::Uuid, prelude::*};

use bevy_persistent::Persistent;
use egui::{Sense, TextEdit, Ui, Widget};
use helium_framework::{
    menu::{Custom, MenuExt},
    prelude::ToastsStorage,
    utils::identifier::Identifier,
    widgets::widget,
};
use rust_i18n::t;

use crate::{
    settings_module::{SettingsModuleStruct, SettingsRegistrationExt}, widgets::dock_button, RizTabPresets, RizliumDockStateMirror
};
pub struct Docking;

impl Plugin for Docking {
    fn build(&self, app: &mut App) {
        app.menu_context(|mut ctx| {
            ctx.with_sub_menu("dock_buttons_menu", "Window".into(), 9, |mut sub| {
                sub.add(
                    "dock_buttons",
                    "_buttons".into(),
                    Custom(Box::new(|u, w, _| widget(w, u, dock_button))),
                    0,
                )
            });
        });
        app.register_settings_module(
            "docking",
            SettingsModuleStruct::new(
                docking_ui_module,
                apply_docking_settings,
                t!("settings.docking"),
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
// a settings module
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
    // show all presets
    egui::ScrollArea::vertical().show(&mut ui, |ui| {
        let current_value = &mut state.selected_preset;
        for (index, (uuid, name, preset)) in state.temp_presets.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if state.current_editing_name == Some(*uuid) {
                    // if we are editing the name, show a text edit
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
                } else if egui::Label::new(name.as_str())
                    .sense(Sense::all())
                    .ui(ui)
                    .on_hover_text("Double click to edit")
                    .double_clicked()
                {
                    state.current_editing_name = Some(*uuid);
                    changed = true;
                }
                // right aligned button to delete or select the preset
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
    // add a button to create a new preset
    if ui.button("Add new preset").clicked() {
        let new_name = format!("Preset {}", state.temp_presets.len() + 1);
        // remove the settings tab from current
        let mut current = current.clone();
        current
            .main_surface_mut()
            .retain_tabs(|tab| tab != &mut Identifier::from("settings.tab"));
        state.temp_presets.push((Uuid::new_v4(), new_name, current));
        changed = true;
    }
    if let Some(index) = to_delete_index {
        // remove the preset from the list
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
    // apply current preset to the docking state
    if let Some(preset) = storage
        .temp_presets
        .iter()
        .find(|(id, _, _)| *id == storage.selected_preset)
    {
        current.0 = Some(preset.2.clone());
    } else {
        warn!("Selected a non-existing docking preset");
    }
    // save presets to persistent storage
    if let Err(e) = presets.set(storage.temp_presets) {
        error!("Failed to save docking presets: {}", e);
        toast.error(t!("settings.docking.save_error"));
    } else {
        info!("Docking presets saved successfully");
    }
}
