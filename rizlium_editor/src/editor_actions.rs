
use bevy::ecs::system::{SystemBuffer, SystemMeta, SystemParam};
use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use rizlium_render::{LoadChartEvent, ShowLines, TimeControlEvent};
use serde::de::DeserializeOwned;
use serde::Serialize;
use crate::{files::open_dialog, files::PendingDialog, RecentFiles};

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
