use std::borrow::{Borrow, Cow};

use bevy::{
    app::{Plugin, Update},
    ecs::{
        resource::Resource,
        system::{In, Res, ResMut},
    },
};
use bevy_persistent::{Persistent, StorageFormat};
use egui::Ui;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::settings_module::{SettingsModuleStruct, SettingsRegistrationExt};

pub struct I18nPlugin;

impl Plugin for I18nPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        // todo: move config_dir to static
        let config_dir = dirs::config_dir()
            .expect("Config dir is None")
            .join("rizlium-editor");
        let a = Persistent::<Locale>::builder()
            .format(StorageFormat::Json)
            .name("Tab layout presets")
            .path(config_dir.join("locale.json"))
            .default(Locale::default())
            .build()
            .expect("failed to setup tab presets");
        rust_i18n::set_locale(a.0.borrow());

        app.register_settings_module(
            "settings.language",
            SettingsModuleStruct::new(language_ui, set_locale, t!("settings.language")),
        );
        app.add_systems(Update, sync_locale);

        app.insert_resource(a);
    }
}

#[derive(Resource, Serialize, Deserialize)]
struct Locale(Cow<'static, str>);

impl Default for Locale {
    fn default() -> Self {
        Self("en".into())
    }
}
fn sync_locale(locale: Res<Persistent<Locale>>) {
    rust_i18n::set_locale(locale.0.borrow());
}

fn language_ui(
    In((mut ui, new_locale)): In<(Ui, Option<Cow<'static, str>>)>,
    locale: Res<Persistent<Locale>>,
) -> Option<Cow<'static, str>> {
    let ui = &mut ui;
    let current = locale.0.clone();
    ui.menu_button(current, |ui| {
        for l in rust_i18n::available_locales!() {
            if ui.button(l).clicked() {
                ui.close_menu();
                return Some(Cow::Borrowed(l));
            }
        }
        None
    })
    .inner
    .flatten()
    .or(new_locale)
}

fn set_locale(In(locale): In<Cow<'static, str>>, mut res: ResMut<Persistent<Locale>>) {
    res.0 = locale;
    res.persist().expect("failed to save config");
}
