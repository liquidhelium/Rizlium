//! Hotkey 实现。
//! 工作方式：多个键时，最后一个键使用 [`TriggerType`] 定义的触发方式，其他键要保持按下。

use bevy::{
    ecs::{schedule::BoxedCondition, system::Command},
    prelude::*,
    utils::HashMap,
    window::PrimaryWindow,
};
use bevy_egui::EguiOutput;
use dyn_clone::DynClone;
use smallvec::SmallVec;

use crate::{ActionId, ActionRegistry};

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
    Released,
    PressAndRelease,
    Repeat,
}

impl TriggerType {
    fn check_trigger(&self, code: KeyCode, input: &mut Input<KeyCode>) -> bool {
        use TriggerType::*;
        let triggered = match self {
            Pressed => input.just_pressed(code),
            Released => input.just_released(code),
            PressAndRelease => input.just_pressed(code) || input.just_released(code),
            Repeat => input.pressed(code),
        };
        input.clear_just_pressed(code);
        input.clear_just_released(code);
        return triggered;
    }
}

pub struct Hotkey {
    pub trigger_type: TriggerType,
    pub trigger_when: BoxedCondition,
    pub key: SmallVec<[KeyCode; 4]>,
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
const fn always() -> bool {
    true
}
impl Hotkey {
    pub fn new<M>(key: impl IntoIterator<Item = KeyCode>, trigger_when: impl Condition<M>) -> Self {
        Self {
            trigger_type: TriggerType::Pressed,
            trigger_when: new_condition(trigger_when),
            key: key.into_iter().collect(),
        }
    }
    pub fn new_global(key: impl IntoIterator<Item = KeyCode>) -> Self {
        Self::new(key, always)
    }
    /// 在应用于 `world` 前一定要先 `initialize`.
    pub fn initialize(&mut self, world: &mut World) {
        self.trigger_when.initialize(world);
    }

    pub fn is_triggered_by_keyboard(&self, world: &mut World) -> bool {
        if self.key.is_empty() {
            return false;
        }
        let mut input = world.resource_mut::<Input<KeyCode>>();
        let mut other_all_pressed = true;
        for code in self.key.iter().copied() {
            other_all_pressed &= input.pressed(code);
        }
        other_all_pressed
            && self
                .trigger_type
                .check_trigger(*self.key.last().unwrap(), &mut *input)
    }
    pub fn should_trigger(&mut self, world: &mut World) -> bool {
        let not_editing_text = !world
            .query_filtered::<&EguiOutput, With<PrimaryWindow>>()
            .get_single(world)
            .map_or(false, |e| e.platform_output.mutable_text_under_cursor);
        let has_modifier = self.key.contains(&KeyCode::AltLeft)
            || self.key.contains(&KeyCode::AltRight)
            || self.key.contains(&KeyCode::ControlLeft)
            || self.key.contains(&KeyCode::ControlRight);
        self.is_triggered_by_keyboard(world)
            && self.trigger_when.run_readonly((), world)
            && (not_editing_text || has_modifier)
    }

    pub fn hotkey_text(&self) -> String {
        self.key
            .iter()
            .map(|k| format!("{k:?}"))
            .collect::<Vec<_>>()
            .join("+")
    }
}

#[derive(Resource, Default, Deref)]
pub struct HotkeyRegistry(HashMap<ActionId, SmallVec<[Hotkey; 3]>>);

pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HotkeyRegistry>();
        app.add_systems(
            PostUpdate,
            dispatch_hotkey.after(bevy_egui::EguiSet::ProcessOutput),
        );
    }
}

fn dispatch_hotkey(world: &mut World) {
    world.resource_scope(|world: &mut World, mut hotkeys: Mut<'_, HotkeyRegistry>| {
        for (id, listeners) in hotkeys.0.iter_mut() {
            for listener in listeners {
                if listener.should_trigger(world) {
                    // todo: error handling
                    world
                        .resource_scope(
                            |world: &mut World, mut actions: Mut<'_, ActionRegistry>| {
                                actions.run_instant(id, (), world)
                            },
                        )
                        .expect("encountered err (todo handle this)");
                }
            }
        }
    });
}

pub trait HotkeysExt {
    fn register_hotkey(
        &mut self,
        id: impl Into<ActionId>,
        hotkeys: impl IntoIterator<Item = Hotkey>,
    ) -> &mut Self;
}

impl HotkeysExt for App {
    fn register_hotkey(
        &mut self,
        id: impl Into<ActionId>,
        hotkey_list: impl IntoIterator<Item = Hotkey>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world: &mut World, mut hotkeys: Mut<'_, HotkeyRegistry>| {
                let mut hotkey_list: SmallVec<[Hotkey; 3]> = hotkey_list
                    .into_iter()
                    .map(|mut k| {
                        k.initialize(world);
                        k
                    })
                    .collect();
                let listeners = hotkeys.0.entry(id.into()).or_default();
                listeners.append(&mut hotkey_list);
            });
        self
    }
}
