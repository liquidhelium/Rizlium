
use bevy::{
    ecs::{
        schedule::BoxedCondition,
        system::{SystemParam, SystemState},
    },
    prelude::*,
    utils::HashMap,
};
use egui::Ui;
use snafu::Snafu;

use crate::utils::dot_path::DotPath;
pub trait TabProvider: SystemParam + Send + Sync {
    fn ui(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, has_focus: bool);
    fn name() -> String {
        // TODO: i18n
        default()
    }
    fn avaliable(_world: &World) -> bool {
        true
    }
}

pub struct TabInstace<T: TabProvider + 'static> {
    state: Option<SystemState<T>>,
}

impl<T: TabProvider + 'static> Default for TabInstace<T> {
    fn default() -> Self {
        Self { state: None }
    }
}

pub trait CachedTab: Send + Sync {
    fn ui(&mut self, world: &mut World, ui: &mut Ui, has_focus: bool);
    fn name(&self) -> String;
    fn avaliable(&self, world: &World) -> bool;
}

impl<T: TabProvider> CachedTab for TabInstace<T> {
    fn ui(&mut self, world: &mut World, ui: &mut Ui, has_focus: bool) {
        let mut state = self
            .state
            .take()
            .unwrap_or_else(|| SystemState::<T>::from_world(world));
        T::ui(world, &mut state, ui, has_focus);
        state.apply(world);
        if self.state.is_none() {
            self.state = Some(state);
        }
    }
    fn name(&self) -> String {
        T::name()
    }
    fn avaliable(&self, world: &World) -> bool {
        T::avaliable(world)
    }
}

pub type TabId = DotPath;

pub struct TabStorage {
    boxed: Box<dyn System<In = &'static mut Ui, Out = ()>>,
    avalible_condition: BoxedCondition,
    tab_title: &'static str,
}

impl TabStorage {
    pub fn run_with(&mut self, world: &mut World, ui: &mut Ui) -> TabResult {
        unsafe {
            self.avalible_condition
                .run_readonly((), world)
                .then(|| {
                    self.boxed.run(&mut *(ui as *mut Ui), world);
                    self.boxed.apply_deferred(world)
                })
                .ok_or(TabError::NotAvalible {
                    name: self.tab_title,
                })
        }
    }
    pub fn title(&self) -> &'static str {
        self.tab_title
    }
}
pub type TabResult = Result<(), TabError>;

#[derive(Snafu, Debug)]
pub enum TabError {
    #[snafu(display("Tab {name} is not avalible."))]
    NotAvalible { name: &'static str },
}

#[derive(Resource, Deref, Default)]
pub struct TabRegistry(HashMap<DotPath, TabStorage>);

impl TabRegistry {
    pub fn tab_ui(&mut self, ui: &mut Ui, world: &mut World, tab: &TabId) {
        use egui::{Color32, RichText};

        if let Some(tab) = self.0.get_mut(tab) {
            let Ok(()) = tab.run_with(world, ui) else {
                ui.colored_label(Color32::GRAY, RichText::new("Not avalible").italics());
                return;
            };
        } else {
            ui.colored_label(Color32::RED, format!("Tab {tab} does not exist."));
        }
    }
}

pub trait TabRegistrationExt {
    fn register_tab<M1, M2>(
        &mut self,
        id: TabId,
        name: &'static str,
        system: impl IntoSystem<&'static mut Ui, (), M1>,
        avalible_when: impl Condition<M2>,
    ) -> &mut Self;
}

impl TabRegistrationExt for App {
    fn register_tab<M1, M2>(
        &mut self,
        id: TabId,
        name: &'static str,
        system: impl IntoSystem<&'static mut Ui, (), M1>,
        avalible_when: impl Condition<M2>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world, mut registry: Mut<TabRegistry>| {
                registry.0.insert(
                    id,
                    TabStorage {
                        boxed: Box::new({
                            let mut sys = IntoSystem::into_system(system);
                            sys.initialize(world);
                            sys
                        }),
                        avalible_condition: {
                            let mut sys = new_condition(avalible_when);
                            sys.initialize(world);
                            sys
                        },
                        tab_title: name,
                    },
                )
            });
        self
    }
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

pub struct TabPlugin;

impl Plugin for TabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabRegistry>();
    }
}
