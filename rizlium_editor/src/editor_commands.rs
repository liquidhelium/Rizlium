use bevy::ecs::system::{CommandQueue, SystemBuffer};
use bevy::prelude::*;
use bevy_persistent::Persistent;
use rizlium_render::{TimeControlEvent, LoadChartEvent, ShowLines};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::{PendingDialog, open_dialog, RecentFiles};
#[derive(Default)]
pub struct EditorCommands{
    commands: CommandQueue,
}

impl SystemBuffer for EditorCommands {
    fn apply(&mut self, _system_meta: &bevy::ecs::system::SystemMeta, world: &mut World) {
        self.apply(world);
    }
}



impl EditorCommands {
    pub fn time_control(&mut self,event: TimeControlEvent) {
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
    pub fn apply(&mut self, world: &mut World) {
        self.commands.apply(world);
    }
    pub fn update_recent(&mut self, path: String) {
        self.commands.push(move |world: &mut World| {
            let mut recent = world.resource_mut::<Persistent<RecentFiles>>();
            recent.push(path);
            recent.persist().unwrap();
        });
    }
    pub fn persist_resource<T: Resource+ Serialize + DeserializeOwned>(&mut self) {
        self.commands.push(|world: &mut World| {
            world.resource_mut::<Persistent<T>>().persist().unwrap();
        });
    }
}

pub struct GameConfigure<'c> {
    pub commands: &'c mut EditorCommands
}

impl GameConfigure<'_> {
    pub fn show_line(self, show: Option<usize>)-> Self {
        self.commands.commands.push(move |world:&mut World | {
            if let Some(mut res) = world.get_resource_mut::<ShowLines>() {
                res.0 = show
            }
            else {
                error!("failed to get resource!")
            }
        });
        self
    }
}