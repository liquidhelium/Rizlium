use std::{borrow::Cow, marker::PhantomData};

use bevy::{
    app::{App, Plugin},
    ecs::{
        system::{In, InMut, IntoSystem, Local, ReadOnlySystem, System},
        resource::Resource,
        world::{Mut, World},
    },
    log::error,
    prelude::Deref,
};
use egui::{Align, Button, CentralPanel, Layout, ScrollArea, SidePanel, Ui};
use indexmap::IndexMap;
use rust_i18n::t;

use helium_framework::{prelude::*, utils::identifier::Identifier};

pub trait SettingsRegistrationExt {
    fn register_settings_module(
        &mut self,
        id: impl Into<Identifier>,
        module: impl SettingsModule,
    ) -> &mut Self;
}

impl SettingsRegistrationExt for App {
    fn register_settings_module(
        &mut self,
        id: impl Into<Identifier>,
        module: impl SettingsModule,
    ) -> &mut Self {
        let v = Box::new(SettingsModuleDyn::from_module(module, self.world_mut()));
        self.world_mut()
            .resource_mut::<SettingsModuleRegistry>()
            .0
            .insert(id.into(), v);
        self
    }
}

pub struct SettingsPlugin;

impl Plugin for SettingsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<SettingsModuleRegistry>();
    }
    fn finish(&self, app: &mut App) {
        app.register_tab("settings", t!("settings.tab"), settings_tab, || true);
    }
}

fn settings_tab(InMut( ui): InMut<Ui>, world: &mut World, mut opened_tab: Local<usize>) {

    world.resource_scope(
        |world: &mut World, mut registry: Mut<SettingsModuleRegistry>| {
            ui.heading("Settings");
            ui.scope(|ui| {
                SidePanel::left("settings_entry")
                    .min_width(60.)
                    .max_width(80.)
                    .show_inside(ui, |ui| {
                        ScrollArea::new([false, true]).show(ui, |ui| {
                            for (i, runner) in registry.0.values_mut().enumerate() {
                                if ui
                                    .selectable_label(i == *opened_tab, runner.name())
                                    .clicked()
                                {
                                    *opened_tab = i
                                }
                            }
                        });
                    });
                CentralPanel::default().show_inside(ui, |ui| {
                    if let Some((_, runner)) = registry.0.get_index_mut(*opened_tab) {
                        runner.run_ui_system(ui, world);
                        let has_mutation = runner.has_mutation();
                        ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                            if ui.add_enabled(has_mutation, Button::new("Apply")).clicked() {
                                runner.run_apply_system(world);
                            }
                        });
                    }
                })
            });
        },
    );
}

#[derive(Resource, Default, Deref)]
pub struct SettingsModuleRegistry(IndexMap<Identifier, Box<dyn ModuleRunner>>);

pub struct SettingsModuleDyn<Storage: Send + Sync + 'static> {
    storage: Option<Storage>,
    ui_system: Box<dyn ReadOnlySystem<In = In<(Ui, Option<Storage>)>, Out = Option<Storage>>>,
    apply_edit_system: Box<dyn System<In = In<Storage>, Out = ()>>,
    name: Cow<'static, str>,
}

pub trait ModuleRunner: Send + Sync + 'static {
    fn run_ui_system(&mut self, ui: &mut Ui, world: &World);
    fn run_apply_system(&mut self, world: &mut World);
    fn has_mutation(&self) -> bool;
    fn name(&self) -> Cow<'static, str>;
}

impl<Storage: Send + Sync + 'static> ModuleRunner for SettingsModuleDyn<Storage> {
    fn run_apply_system(&mut self, world: &mut World) {
        if let Some(storage) = self.storage.take() {
            self.apply_edit_system.run(storage, world);
        } else {
            error!(
                "Can't apply edit because it hasn't been initialized. system to run: {}",
                self.apply_edit_system.name()
            );
        }
    }
    fn run_ui_system(&mut self, ui: &mut Ui, world: &World) {
        let child = ui.child_ui(ui.max_rect(), *ui.layout(), None);
        self.storage = self
            .ui_system
            .run_readonly((child, self.storage.take()), world);
    }
    fn has_mutation(&self) -> bool {
        self.storage.is_some()
    }
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}
impl<S: Send + Sync + 'static> SettingsModuleDyn<S> {
    fn from_module(
        module: impl SettingsModule<SettingsTempStorage = S>,
        world: &mut World,
    ) -> Self {
        Self {
            storage: None,
            ui_system: module.ui_system(world),
            apply_edit_system: module.apply_edit_system(world),
            name: module.name(),
        }
    }
}

pub trait SettingsModule {
    type SettingsTempStorage: Send + Sync + 'static;
    fn ui_system(
        &self,
        world: &mut World,
    ) -> Box<
        dyn ReadOnlySystem<
            In = In<(Ui, Option<Self::SettingsTempStorage>)>,
            Out = Option<Self::SettingsTempStorage>,
        >,
    >;
    fn apply_edit_system(
        &self,
        world: &mut World,
    ) -> Box<dyn System<In = In<Self::SettingsTempStorage>, Out = ()>>;
    fn name(&self) -> Cow<'static, str>;
}

pub struct SettingsModuleStruct<Storage, Q, R, M2, M3>
where
    Q: IntoSystem<In<(Ui, Option<Storage>)>, Option<Storage>, M2> + Clone,
    Q::System: ReadOnlySystem,
    R: IntoSystem<In<Storage>, (), M3> + Clone,
    Storage: Send + Sync + 'static,
{
    ui_system: Q,
    apply_edit_system: R,
    name: Cow<'static, str>,
    _phantom: PhantomData<(Storage, M2, M3)>,
}

impl<Storage, Q, R, M2, M3> SettingsModuleStruct<Storage, Q, R, M2, M3>
where
    Q: IntoSystem<In<(Ui, Option<Storage>)>, Option<Storage>, M2> + Clone,
    Q::System: ReadOnlySystem,
    R: IntoSystem<In<Storage>, (), M3> + Clone,
    Storage: Send + Sync,
{
    pub fn new(ui_system: Q, apply_edit_system: R, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            ui_system,
            apply_edit_system,
            name: name.into(),
            _phantom: PhantomData,
        }
    }
}

impl<Storage, Q, R, M2, M3> SettingsModule for SettingsModuleStruct<Storage, Q, R, M2, M3>
where
    Q: IntoSystem<In<(Ui, Option<Storage>)>, Option<Storage>, M2> + Clone,
    Q::System: ReadOnlySystem,
    R: IntoSystem<In<Storage>, (), M3> + Clone,
    Storage: Send + Sync + 'static,
{
    type SettingsTempStorage = Storage;
    fn ui_system(
        &self,
        world: &mut World,
    ) -> std::boxed::Box<
        (dyn bevy::prelude::ReadOnlySystem<
            In = In<(egui::Ui, std::option::Option<Storage>)>,
            Out = std::option::Option<Storage>,
        > + 'static),
    > {
        let mut system = IntoSystem::into_system(self.ui_system.clone());
        system.initialize(world);
        Box::new(system)
    }
    fn apply_edit_system(
        &self,
        world: &mut World,
    ) -> Box<dyn System<Out = (), In = In<Self::SettingsTempStorage>>> {
        let mut system = IntoSystem::into_system(self.apply_edit_system.clone());
        system.initialize(world);
        Box::new(system)
    }
    fn name(&self) -> Cow<'static, str> {
        self.name.clone()
    }
}
