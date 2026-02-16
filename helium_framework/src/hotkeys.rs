//! Hotkey 实现。
//! 工作方式：多个键时，最后一个键使用 [`TriggerType`] 定义的触发方式，其他键要保持按下。

use bevy::platform::collections::HashMap;
use bevy::{ecs::schedule::BoxedCondition, log::*, prelude::*, window::PrimaryWindow};
use bevy_egui::EguiOutput;
use smallvec::SmallVec;

use crate::prelude::{ActionError, ActionId, RSystemRegistry, ReflectSystemRunner};
use crate::utils::new_condition;
pub enum TriggerType {
    Pressed,
    Released,
    PressAndRelease,
    Repeat,
}

pub struct RuntimeTrigger {
    pub trigger_type: RuntimeTriggerType,
    pub code: KeyCode,
}

#[derive(Clone, Copy, Reflect, Debug)]
pub enum RuntimeTriggerType {
    Pressed,
    Pressing,
    Released,
}

impl RuntimeTrigger {
    pub fn pressed(code: KeyCode) -> Self {
        Self {
            trigger_type: RuntimeTriggerType::Pressed,
            code,
        }
    }
    pub fn pressing(code: KeyCode) -> Self {
        Self {
            trigger_type: RuntimeTriggerType::Pressing,
            code,
        }
    }
    pub fn released(code: KeyCode) -> Self {
        Self {
            trigger_type: RuntimeTriggerType::Released,
            code,
        }
    }
    pub fn is_pressed(&self) -> bool {
        matches!(self.trigger_type, RuntimeTriggerType::Pressed)
    }
    pub fn is_pressing(&self) -> bool {
        matches!(self.trigger_type, RuntimeTriggerType::Pressing)
    }
    pub fn is_released(&self) -> bool {
        matches!(self.trigger_type, RuntimeTriggerType::Released)
    }
}

impl TriggerType {
    fn check_trigger(
        &self,
        code: KeyCode,
        input: &mut ButtonInput<KeyCode>,
    ) -> Option<RuntimeTrigger> {
        use TriggerType::*;
        let runtime_trigger = match self {
            Pressed if input.just_pressed(code) => Some(RuntimeTrigger::pressed(code)),
            Released if input.just_released(code) => Some(RuntimeTrigger::released(code)),
            PressAndRelease => input
                .just_pressed(code)
                .then_some(RuntimeTrigger::pressed(code))
                .or_else(|| {
                    input
                        .just_released(code)
                        .then_some(RuntimeTrigger::released(code))
                }),
            Repeat if input.pressed(code) => Some(RuntimeTrigger::pressing(code)),
            _ => None,
        };
        if input.just_released(code) {
            debug!("just released {code:?};");
        }
        input.clear_just_pressed(code);
        input.clear_just_released(code);
        runtime_trigger
    }
}

pub struct Hotkey {
    pub trigger_type: TriggerType,
    pub trigger_when: BoxedCondition,
    pub key: SmallVec<[KeyCode; 4]>,
}
const fn always() -> bool {
    true
}
impl Hotkey {
    pub fn new<M>(key: impl IntoIterator<Item = KeyCode>, trigger_when: impl SystemCondition<M>) -> Self {
        Self::new_advanced(key, trigger_when, TriggerType::Pressed)
    }
    pub fn new_advanced<M>(
        key: impl IntoIterator<Item = KeyCode>,
        trigger_when: impl SystemCondition<M>,
        trigger_type: TriggerType,
    ) -> Self {
        Self {
            trigger_type,
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

    pub fn keyboard_trigger(&self, world: &mut World) -> Option<RuntimeTrigger> {
        if self.key.is_empty() {
            return None;
        }
        let mut input = world.resource_mut::<ButtonInput<KeyCode>>();
        let mut other_all_pressed = true;
        for code in self.key.iter().take(self.key.len() - 1).copied() {
            other_all_pressed &= input.pressed(code);
            if !other_all_pressed {
                break;
            }
        }
        other_all_pressed
            .then(|| {
                self.trigger_type
                    .check_trigger(*self.key.last().unwrap(), &mut input)
            })
            .flatten()
    }
    pub fn trigger_result(&mut self, world: &mut World) -> Option<RuntimeTrigger> {
        let not_editing_text = !world
            .query_filtered::<&EguiOutput, With<PrimaryWindow>>()
            .single(world)
            .is_ok_and(|e| e.platform_output.mutable_text_under_cursor);
        let has_modifier = self.key.contains(&KeyCode::AltLeft)
            || self.key.contains(&KeyCode::AltRight)
            || self.key.contains(&KeyCode::ControlLeft)
            || self.key.contains(&KeyCode::ControlRight);

        (self.trigger_when.run_readonly((), world).unwrap_or(false) && (not_editing_text || has_modifier))
            .then(|| self.keyboard_trigger(world))
            .flatten()
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
            dispatch_hotkey.after(bevy_egui::EguiPostUpdateSet::ProcessOutput),
        );
    }
}
enum Either {
    HasTrigger(
        ReflectSystemRunner<'static, In<RuntimeTrigger>, ()>,
        RuntimeTrigger,
    ),
    NoTrigger(ReflectSystemRunner<'static, (), ()>),
}

impl From<ReflectSystemRunner<'static, (), ()>> for Either {
    fn from(v: ReflectSystemRunner<'static, (), ()>) -> Self {
        Self::NoTrigger(v)
    }
}

impl Either {
    fn run(self, world: &mut World) -> Result<(), ActionError> {
        match self {
            Either::HasTrigger(runner, t) => {
                runner.run(world, In(t))?;
            }
            Either::NoTrigger(runner) => {
                runner.run(world, ())?;
            }
        }
        Ok(())
    }
}

fn dispatch_hotkey(world: &mut World) {
    let mut triggered = vec![];
    world.resource_scope(|world: &mut World, mut hotkeys: Mut<'_, HotkeyRegistry>| {
        // 收集所有快捷键并按键数降序排序，确保更具体的快捷键（如 Ctrl+Shift+S）优先于较短的（如 Ctrl+S）被检查
        let mut all_hotkeys: Vec<(&ActionId, &mut Hotkey)> = hotkeys
            .0
            .iter_mut()
            .flat_map(|(id, listeners)| listeners.iter_mut().map(move |l| (id, l)))
            .collect();

        all_hotkeys.sort_by(|a, b| b.1.key.len().cmp(&a.1.key.len()));

        for (id, listener) in all_hotkeys {
            if let Some(trigger) = listener.trigger_result(world) {
                let actions = world.resource::<RSystemRegistry>();
                match (actions.construct_runner(id), actions.construct_runner(id)) {
                    (Ok(runner), Err(_)) => {
                        info!("Hotkey {id} triggered");
                        triggered.push(Either::HasTrigger(runner, trigger));
                    }
                    (Err(_), Ok(runner)) => {
                        info!("Hotkey {id} triggered without trigger");
                        triggered.push(Either::NoTrigger(runner));
                    }
                    (Err(e1), Err(e2)) => {
                        error!("Failed to construct hotkey action runner for {id}: {e1:?}, {e2:?}");
                    }
                    (Ok(_), Ok(_)) => {
                        error!("Both triggered and non-triggered action runners exist for {id}, this is not allowed.");
                    }
                }
            }
        }
    });
    for runner in triggered {
        if let Err(e) = runner.run(world) {
            error!("Failed to run hotkey action: {e:?}");
        }
    }
}

pub trait HotkeysExt {
    fn register_hotkey(
        &mut self,
        id: impl Into<ActionId>,
        hotkeys: impl IntoIterator<Item = Hotkey>,
    ) -> &mut Self;
}

impl HotkeysExt for World {
    fn register_hotkey(
        &mut self,
        id: impl Into<ActionId>,
        hotkey_list: impl IntoIterator<Item = Hotkey>,
    ) -> &mut Self {
        self.resource_scope(|world: &mut World, mut hotkeys: Mut<'_, HotkeyRegistry>| {
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

impl HotkeysExt for App {
    fn register_hotkey(
        &mut self,
        id: impl Into<ActionId>,
        hotkey_list: impl IntoIterator<Item = Hotkey>,
    ) -> &mut Self {
        self.world_mut().register_hotkey(id, hotkey_list);
        self
    }
}
