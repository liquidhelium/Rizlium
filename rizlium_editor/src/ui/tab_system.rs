pub(crate) mod canvas_window;

pub(crate) mod file_menu;

pub(crate) mod game_view;

pub(crate) mod information;

pub(crate) mod show_line_control;

mod note_edit;
pub(crate) mod spline_edit;

pub mod tabs {
    pub use super::canvas_window::CanvasWindow;
    pub use super::file_menu::FileMenu;
    pub use super::game_view::GameViewTab;
    pub use super::information::information;
    pub use super::note_edit::NoteWindow;
    pub use super::show_line_control::ShowLineControl;
    pub use super::spline_edit::SplineWindow;
}



use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use egui::Ui;
use leafwing_input_manager::{
    prelude::{InputMap, InputManagerPlugin}, Actionlike, InputManagerBundle,
};

pub trait TabProvider: SystemParam + Send + Sync {
    type Hotkey: Actionlike;
    fn ui(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui, has_focus: bool);
    fn name() -> String {
        // TODO: i18n
        default()
    }
    fn avaliable(_world: &World) -> bool {
        true
    }
    fn default_map() -> InputMap<Self::Hotkey> {
        default()
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
    fn init_hotkey(&self, app: &mut App);
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
    fn init_hotkey(&self, app: &mut App) {
        app.world.spawn(InputManagerBundle::<T::Hotkey> {
            input_map: T::default_map(),
            ..default()
        });
        if !app.is_plugin_added::<InputManagerPlugin<T::Hotkey>>() {
            app.add_plugins(InputManagerPlugin::<T::Hotkey>::default());
        }
    }
}
