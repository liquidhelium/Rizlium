pub(crate) mod canvas_window;

pub(crate) mod file_menu;

pub(crate) mod game_view;

pub(crate) mod information;

pub(crate) mod show_line_control;

pub(crate) mod spline_edit;
mod note_edit;

pub mod tabs {
    pub use super::canvas_window::CanvasWindow;
    pub use super::file_menu::FileMenu;
    pub use super::game_view::GameViewTab;
    pub use super::information::information;
    pub use super::show_line_control::ShowLineControl;
    pub use super::spline_edit::SplineWindow;
    pub use super::note_edit::NoteWindow;
}

use bevy::{
    ecs::system::{SystemParam, SystemState},
    prelude::*,
};
use egui::Ui;
pub trait TabProvider: SystemParam + Send + Sync {
    fn system(world: &mut World, state: &mut SystemState<Self>, ui: &mut Ui);
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
    fn ui(&mut self, world: &mut World, ui: &mut Ui);
    fn name(&self) -> String;
    fn avaliable(&self, world: &World) -> bool;
}

impl<T: TabProvider> CachedTab for TabInstace<T> {
    fn ui(&mut self, world: &mut World, ui: &mut Ui) {
        let mut state = self
            .state
            .take()
            .unwrap_or_else(|| SystemState::<T>::from_world(world));
        T::system(world, &mut state, ui);
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
