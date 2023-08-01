use bevy::prelude::Resource;
use egui_dock::Tree;
use ui::RizliumTab;
pub use ui::*;
mod ui;
#[derive(Debug, Resource, Default)]
pub struct EditorState {
    pub debug_resources: DebugResources,
}
#[derive(Debug, Default)]
pub struct DebugResources {
    pub show_cursor: bool
}

#[derive(Debug,Resource)]
pub struct RizDockTree {
    pub tree: Tree<RizliumTab>,
}

impl Default for RizDockTree {
    fn default() -> Self {
        Self { tree: Tree::new(vec![RizliumTab::GameView]) }
    }
}