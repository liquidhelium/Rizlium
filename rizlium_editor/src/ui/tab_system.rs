use bevy::{ecs::schedule::BoxedCondition, prelude::*, utils::HashMap};
use egui::Ui;
use snafu::Snafu;

use crate::{utils::{dot_path::DotPath, new_condition}, RizDockState};
pub type TabId = DotPath;

pub struct TabStorage {
    boxed: Box<dyn System<In = &'static mut Ui, Out = ()>>,
    avalible_condition: BoxedCondition,
    tab_title: &'static str,
}

#[derive(Resource, Default, PartialEq, Eq)]
pub struct FocusedTab(pub Option<DotPath>);

pub fn tab_focused(tab: impl Into<DotPath>) -> impl Condition<()> {
    resource_exists_and_equals(FocusedTab(Some(tab.into()))).and_then(|| true)
}

pub fn tab_opened(tab: impl Into<DotPath>) -> impl Condition<()> {
    let tab = tab.into();
    (move |res: Option<Res<RizDockState>>| {
        res.is_some_and(|res| res.state.find_tab(&tab).is_some())
    })
    .and_then(|| true)
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
        id: impl Into<TabId>,
        name: &'static str,
        system: impl IntoSystem<&'static mut Ui, (), M1>,
        avalible_when: impl Condition<M2>,
    ) -> &mut Self;
}

impl TabRegistrationExt for App {
    fn register_tab<M1, M2>(
        &mut self,
        id: impl Into<TabId>,
        name: &'static str,
        system: impl IntoSystem<&'static mut Ui, (), M1>,
        avalible_when: impl Condition<M2>,
    ) -> &mut Self {
        self.world
            .resource_scope(|world, mut registry: Mut<TabRegistry>| {
                registry.0.insert(
                    id.into(),
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
pub struct TabPlugin;

impl Plugin for TabPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TabRegistry>()
            .init_resource::<FocusedTab>();
    }
}
