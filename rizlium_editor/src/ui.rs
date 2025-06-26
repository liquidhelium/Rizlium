use bevy::{
    asset::uuid::Uuid,
    ecs::{
        change_detection::{DetectChanges, DetectChangesMut}, schedule::Condition, system::{Res, ResMut}
    },
    log::debug,
    prelude::{Deref, DerefMut, Resource},
};

use bevy_persistent::Persistent;
use egui_dock::{DockState, NodeIndex, Tree};

pub mod widgets;
use helium_framework::prelude::TabId;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize, Default, DerefMut, Deref, Clone)]
pub struct RizTabPresets(Vec<(Uuid, String, DockState<TabId>)>);

#[derive(Resource, Deref, DerefMut, Default, Debug)]
pub struct RizliumDockStateMirror(pub Option<DockState<TabId>>);

pub fn sync_dock_state(
    mut dock_state: ResMut<Persistent<RizliumDockState>>,
    mut mirror: ResMut<RizliumDockStateMirror>,
) {
    if mirror.is_changed() {
        if let Some(mirror_state) = &mirror.0 {
            debug!("Syncing mirror state");
            dock_state.bypass_change_detection().0 = mirror_state.clone();
        }
    } else {
        mirror.bypass_change_detection().0 = Some(dock_state.0.clone());
    }
}

#[derive(Resource, Debug, Clone, Serialize, Deserialize)]
pub struct RizliumDockState(pub DockState<TabId>);

impl Default for RizliumDockState {
    fn default() -> Self {
        // pretty hacky way to create an empty dock state...
        // we create a tree with no nodes and then set it as the main surface
        let mut dock_state = DockState::new(vec![]);

        *dock_state.main_surface_mut() = Tree::default();
        Self(dock_state)
    }
}
pub fn tab_opened(tab: impl Into<TabId>) -> impl Condition<()> {
    let tab = tab.into();
    (move |res: Option<Res<RizliumDockStateMirror>>| res.is_some_and(|res| res.0.as_ref().is_some_and(|r| r.find_tab(&tab).is_some())))
        .and(|| true)
}
