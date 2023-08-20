use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_persistent::Persistent;
use egui::{Color32, RichText};

use crate::{RizDockTree, RizTabPresets};

use super::WidgetSystem;

#[derive(SystemParam)]
pub struct PresetButtons<'w> {
    presets: ResMut<'w, Persistent<RizTabPresets>>,
    tree: ResMut<'w, RizDockTree>,
}

impl WidgetSystem for PresetButtons<'static> {
    type Extra<'a> = &'a mut bool;
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        open_window: Self::Extra<'_>,
    ) {
        let PresetButtons::<'_> {
            mut presets,
            mut tree,
        } = state.get_mut(world);
        for (key, preset_tree) in presets.get_mut().iter_mut() {
            if ui.button(&*key).clicked() {
                tree.tree = preset_tree.clone();
                ui.close_menu();
            }
        }
        if presets.get().is_empty() {
            ui.label(
                RichText::new("Nothing here..")
                    .color(Color32::GRAY)
                    .italics(),
            );
        }
        if ui.button("Edit..").clicked() {
            *open_window = true;
            ui.close_menu();
        }
        if ui
            .add_enabled(!*open_window, egui::Button::new("Save current as preset"))
            .clicked()
        {
            presets
                .update(|presets| {
                    presets.push(("New".into(), tree.tree.clone()));
                })
                .unwrap();
            ui.close_menu();
        }
    }
}

#[derive(SystemParam)]
pub struct LayoutPresetEdit<'w, 's> {
    presets: ResMut<'w, Persistent<RizTabPresets>>,
    editing_entry: Local<'s, Option<usize>>,
}

impl WidgetSystem for LayoutPresetEdit<'static, 'static> {
    type Extra<'a> = ();
    fn system(
        world: &mut World,
        state: &mut bevy::ecs::system::SystemState<Self>,
        ui: &mut egui::Ui,
        _extra: Self::Extra<'_>,
    ) {
        let LayoutPresetEdit::<'_, '_> {
            mut presets,
            mut editing_entry,
        } = state.get_mut(world);
        let mut to_remove = None;
        for (index, (key, _)) in presets.get_mut().iter_mut().enumerate() {
            ui.horizontal(|ui| {
                if *editing_entry == Some(index) {
                    ui.text_edit_singleline(key);
                } else if ui.button(&*key).clicked() {
                    *editing_entry = Some(index);
                }
                if ui.small_button("D").clicked() {
                    if *editing_entry == Some(index) {
                        *editing_entry = None;
                    }
                    to_remove = Some(index);
                }
            });
        }
        if presets.get().is_empty() {
            ui.label("No presets stored.");
        }
        if let Some(to_remove) = to_remove {
            presets.get_mut().remove(to_remove);
        }
    }
}
