use bevy::{ecs::system::SystemParam, prelude::*};
use leafwing_input_manager::{
    action_state::ActionData,
    dynamic_action::{DynAction, DynActionMarker, DynActionRegistry},
    prelude::{ActionState, InputManagerPlugin},
};


pub struct HotkeyPlugin;

impl Plugin for HotkeyPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_plugins(InputManagerPlugin::<DynAction>::default())
        .insert_resource(DynActionRegistry::get().unwrap());
    }
    fn finish(&self, app: &mut App) {
        app.world.remove_resource::<DynActionRegistry>().unwrap().finish();
    }
}

pub trait ToDynAction {
    fn to_dyn_action(&self) -> DynAction;
}

impl<T: DynActionMarker> ToDynAction for T {
    fn to_dyn_action(&self) -> DynAction {
        Self::get_action()
    }
}

impl From<Box<dyn ToDynAction>> for DynAction {
    fn from(value: Box<dyn ToDynAction>) -> Self {
        value.to_dyn_action()
    }
}

#[macro_export]
macro_rules! as_dyn_actions {
    ($vis:vis enum $name:ident {
        $($single_action:ident),+
    }) => {
        #[allow(non_snake_case)]
        $vis mod $name {
            use leafwing_input_manager::dynamic_action::DynActionMarker;
            use crate::hotkeys::ToDynAction;
            $(#[derive(DynActionMarker, Default)]
            pub struct $single_action;)+
            pub fn varients() -> impl Iterator<Item = Box<dyn ToDynAction>>{
                [
                    $(Box::<$single_action>::default() as Box<dyn ToDynAction>),+
                ].into_iter()
            }
            pub fn register_varients(app: &mut bevy::prelude::App) {
                use leafwing_input_manager::dynamic_action::RegisterActionToAppExt;
                $(app.register_action::<$single_action>();)+
            }
        }

    };
}

as_dyn_actions! {
    pub enum GlobalEditorAction {
        OpenChartDialog,
        TogglePause,
        SaveDocument
    }
}

#[derive(Component)]
struct EditorHotkey;

#[derive(SystemParam)]
pub struct HotkeyContext<'w, 's> {
    query: Query<'w, 's, &'static ActionState<DynAction>, With<EditorHotkey>>,
}

impl HotkeyContext<'_, '_> {
    pub fn action(&self, action: impl Into<DynAction>) -> &ActionData {
        self.query.single().action_data(action.into())
    }
}
