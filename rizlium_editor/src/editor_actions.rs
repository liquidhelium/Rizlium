use std::sync::Arc;

use bevy::ecs::system::{CommandQueue, SystemBuffer, SystemMeta, SystemParam};
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_persistent::Persistent;
use egui::mutex::Mutex;
use rizlium_render::{LoadChartEvent, ShowLines, TimeControlEvent};
use serde::de::DeserializeOwned;
use serde::Serialize;
use snafu::Snafu;

use crate::{files::open_dialog, files::PendingDialog, RecentFiles};

pub type BoxedStorage = Box<dyn DynActionStorage>;

pub type ActionId = String;

#[derive(Resource, Default)]
pub struct ActionStorages(HashMap<ActionId, BoxedStorage>);

impl ActionStorages {
    pub fn run_instant(
        &mut self,
        id: &str,
        input: impl Reflect,
        world: &mut World,
    ) -> Result<(), ActionError> {
        self.0.get(id).ok_or(ActionError::NotFound { id: id.into() })?.get_command(Box::new(input)).expect("input type mismatch")(world);
        Ok(())
    }
}

pub trait DynActionStorage: Send + Sync {
    fn get_command(
        &self,
        input: Box<dyn Reflect>,
    ) -> Option<Box<dyn FnOnce(&mut World) + Send + Sync + 'static>>;
}

pub struct ActionStorage<Input: Reflect> {
    action: Arc<Mutex<Box<dyn System<In = Input, Out = ()>>>>,
}

impl<Input: Reflect> DynActionStorage for ActionStorage<Input> {
    fn get_command(
        &self,
        input: Box<dyn Reflect>,
    ) -> Option<Box<dyn FnOnce(&mut World) + Send + Sync + 'static>> {
        let owned_action = Arc::clone(&self.action);
        let input = *input.into_any().downcast::<Input>().ok()?;
        Some(Box::new(move |world| {
            owned_action.lock().run(input, world);
        }))
    }
}

#[derive(SystemParam)]
pub struct Actions<'w, 's> {
    commands: Commands<'w, 's>,
    storages: Res<'w, ActionStorages>,
}

impl Actions<'_, '_> {
    pub fn run_action<'id>(
        &mut self,
        id: &'id str,
        input: impl Reflect,
    ) -> Result<(), ActionError> {
        if self.storages.0.contains_key(id) {
            let owned_id = id.to_owned();
            self.commands.add(
                self.storages
                    .0.get(id)
                    .unwrap()
                    .get_command(Box::new(input))
                    .expect("input type mismatch"),
            );
            Ok(())
        } else {
            Err(ActionError::NotFound { id: id.to_string() })
        }
    }
}

#[derive(Snafu, Debug)]
pub enum ActionError {
    #[snafu(display("Action {id} does not exist."))]
    NotFound { id: String },
}

pub trait ActionsExt {
    fn register_action<M, In: FromReflect>(
        &mut self,
        id: &str,
        action: impl IntoSystem<In, (), M>,
    ) -> &mut Self;
}

impl ActionsExt for App {
    fn register_action<M, SystemInput: FromReflect>(
        &mut self,
        id: &str,
        action: impl IntoSystem<SystemInput, (), M>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world, mut actions: Mut<'_, ActionStorages>| {
                let mut system = IntoSystem::into_system(action);
                system.initialize(world);
                actions.0.insert(
                    id.to_string(),
                    Box::new(ActionStorage {
                        action: Arc::new(Mutex::new(Box::new(system))),
                    }),
                );
            });
        self
    }
}

pub struct ActionPlugin;

impl Plugin for ActionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ActionStorages>();
    }
}

#[derive(SystemParam, Deref, DerefMut)]
pub struct EditorCommands<'s> {
    commands: Deferred<'s, ManualEditorCommands>,
}

#[derive(Default)]
pub struct ManualEditorCommands {
    commands: CommandQueue,
}

impl SystemBuffer for ManualEditorCommands {
    fn apply(&mut self, _system_meta: &SystemMeta, world: &mut World) {
        self.commands.apply(world);
    }
}

impl ManualEditorCommands {
    pub fn time_control(&mut self, event: TimeControlEvent) {
        self.commands.push(|world: &mut World| {
            world.send_event(event);
        });
    }
    pub fn load_chart(&mut self, path: String) {
        let dup = path.clone();
        self.commands.push(|world: &mut World| {
            world.send_event(LoadChartEvent(dup));
        });
        self.update_recent(path);
    }
    pub fn open_dialog_and_load_chart(&mut self) {
        self.commands.push(|world: &mut World| {
            let mut res = world.resource_mut::<PendingDialog>();
            open_dialog(&mut res);
        });
    }
    pub fn update_recent(&mut self, path: String) {
        self.commands.push(move |world: &mut World| {
            let mut recent = world.resource_mut::<Persistent<RecentFiles>>();
            recent.push(path);
            recent.persist().unwrap();
        });
    }
    pub fn persist_resource<T: Resource + Serialize + DeserializeOwned>(&mut self) {
        self.commands.push(|world: &mut World| {
            world.resource_mut::<Persistent<T>>().persist().unwrap();
        });
    }

    pub fn apply_manual(&mut self, world: &mut World) {
        self.commands.apply(world);
    }
}

pub struct GameConfigure<'c> {
    pub commands: &'c mut ManualEditorCommands,
}

impl GameConfigure<'_> {
    pub fn show_line(self, show: Option<usize>) -> Self {
        self.commands.commands.push(move |world: &mut World| {
            if let Some(mut res) = world.get_resource_mut::<ShowLines>() {
                res.0 = show
            } else {
                error!("failed to get resource!")
            }
        });
        self
    }
}
