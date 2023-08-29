use bevy::{ecs::system::SystemParam, prelude::*};
use leafwing_input_manager::{
    action_state::ActionData,
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::UserInput,
    Actionlike, InputManagerBundle,
};

use crate::global_actions::{self, GlobalEditorAction};

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<GlobalEditorAction>::default())
            .add_systems(Update, global_actions::dispatch);
        app.world.spawn(InputManagerBundle::<GlobalEditorAction> {
            input_map: GlobalEditorAction::default_map(),
            ..default()
        });
    }
}

#[derive(Actionlike, Reflect, Clone)]
pub enum NoAction {}

#[derive(SystemParam)]
pub struct HotkeyContext<'w, 's, T: Actionlike> {
    query: Query<'w, 's, (&'static ActionState<T>, &'static InputMap<T>)>,
}
use std::ops::Deref;
impl<T: Actionlike> Deref for HotkeyContext<'_, '_, T> {
    type Target = ActionState<T>;
    fn deref(&self) -> &Self::Target {
        self.single().0
    }
}

impl<T: Actionlike> HotkeyContext<'_, '_, T> {
    fn single(&self) -> (&ActionState<T>, &InputMap<T>) {
        self.query
            .get_single()
            .expect("possible calling for T = NoAction, or no action manager found")
    }
}

impl<T: Actionlike> HotkeyContext<'_, '_, T> {
    pub fn iter_inputs(&self) -> impl Iterator<Item = &UserInput> {
        self.single().1.iter_inputs().map(|i| i.iter()).flatten()
    }
}
