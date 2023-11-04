use bevy::{
    ecs::{
        schedule::BoxedCondition,
        system::{Command, SystemParam},
    },
    prelude::*,
};
use dyn_clone::DynClone;
use leafwing_input_manager::{
    prelude::{ActionState, InputManagerPlugin, InputMap},
    user_input::UserInput,
    Actionlike, InputManagerBundle,
};
use smallvec::SmallVec;

use crate::{global_actions::{self, GlobalEditorAction}, ActionId, ActionStorages};

pub trait Action: DynClone + Sync + Send + 'static {
    fn run(&self, world: &mut World);
}

dyn_clone::clone_trait_object!(Action);

impl<T: Clone + Command + Sync> Action for T {
    fn run(&self, world: &mut World) {
        <T as Command>::apply(self.clone(), world)
    }
}

pub enum TriggerType {
    Pressed,
    Relesed,
    PressAndRelease,
    Repeat,
}

impl TriggerType {
    fn check_trigger(&self, code: KeyCode, input: &Input<KeyCode>) -> bool {
        use TriggerType::*;
        match self {
            Pressed => input.just_pressed(code),
            Relesed => input.just_released(code),
            PressAndRelease => input.just_pressed(code) || input.just_released(code),
            Repeat => input.pressed(code),
        }
    }
}

pub struct HotkeyListener {
    pub trigger_type: TriggerType,
    pub trigger_when: BoxedCondition,
    pub key: SmallVec<[KeyCode; 6]>,
    pub action: ActionId
}

fn new_condition<M>(condition: impl Condition<M>) -> BoxedCondition {
    let condition_system = IntoSystem::into_system(condition);
    assert!(
        condition_system.is_send(),
        "Condition `{}` accesses `NonSend` resources. This is not currently supported.",
        condition_system.name()
    );

    Box::new(condition_system)
}
fn always() -> bool {
    true
}
impl HotkeyListener {
    pub fn new<M>(
        action: ActionId,
        key: impl IntoIterator<Item = KeyCode>,
        trigger_when: impl Condition<M>,
    ) -> Self {
        Self {
            trigger_type: TriggerType::Pressed,
            trigger_when: new_condition(trigger_when),
            action,
            key: key.into_iter().collect(),
        }
    }
    pub fn new_global(action: ActionId, key: impl IntoIterator<Item = KeyCode>) -> Self {
        Self::new(action,key, always)
    }
    pub fn initialize(&mut self, world: &mut World) {
        self.trigger_when.initialize(world);
    }
    pub fn trigger<'a>(&'a self, world: &mut World) -> Result<(), crate::ActionError> { // todo: error handling
        world.resource_scope(|world: &mut World, mut actions: Mut<'_, ActionStorages>| {
            actions.run_instant(&self.action, (), world)
        })
    }
    pub fn is_triggered_by_keyboard(&self, world: &World) -> bool {
        if self.key.is_empty() {
            return false;
        }
        let input = world.resource::<Input<KeyCode>>();
        let mut other_all_pressed = true;
        for code in self.key.iter().copied() {
            other_all_pressed &= input.pressed(code);
        }
        other_all_pressed
            && self
                .trigger_type
                .check_trigger(*self.key.last().unwrap(), input)
    }
    pub fn should_trigger(&mut self, world: &World) -> bool {
        self.is_triggered_by_keyboard(world) && self.trigger_when.run_readonly((), world)
    }
}

#[derive(Resource, Default, Deref)]
pub struct HotkeyListeners(Vec<HotkeyListener>);

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(InputManagerPlugin::<GlobalEditorAction>::default())
            .add_systems(Update, global_actions::dispatch);
        app.world.spawn(InputManagerBundle::<GlobalEditorAction> {
            input_map: GlobalEditorAction::default_map(),
            ..default()
        });
        ////
        app.init_resource::<HotkeyListeners>();
        app.add_systems(PreUpdate, dispatch_hotkey);
    }
}

fn dispatch_hotkey(world: &mut World) {
    world.resource_scope(|world: &mut World, mut hotkeys: Mut<'_, HotkeyListeners>| {
        for i in hotkeys.0.iter_mut() {
            if i.should_trigger(world) {
                i.trigger(world).expect("action not found (todo: handle this error) ");
            }
        }
    });
}

pub trait HotkeysExt {
    fn register_hotkey(&mut self, listener: HotkeyListener) -> &mut Self;
}

impl HotkeysExt for App {
    fn register_hotkey(&mut self, mut listener: HotkeyListener) -> &mut Self{
        self.world.resource_scope(|world: &mut World, mut hotkeys: Mut<'_, HotkeyListeners>| {
            listener.initialize(world);
            hotkeys.0.push(listener);
        });
        self
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
