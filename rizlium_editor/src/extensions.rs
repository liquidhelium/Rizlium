pub mod command_panel;
pub mod docking;
mod game;
mod editing;
mod inspector;
pub mod i18n;

use std::borrow::Cow;

use bevy::{asset::{AssetId, Assets}, ecs::world::World, prelude::{App, Deref, DerefMut, Plugin, Resource}, render::{mesh::{Mesh, PrimitiveTopology}, render_asset::RenderAssetUsages}};
use snafu::Snafu;

use crate::menu::{
    Category, ItemAsContainer, ItemGroup, MenuItem, MenuItemProvider, MenuItemVariant, SubMenu,
};

use self::{command_panel::CommandPanel, docking::Docking, editing::Editing, game::Game, i18n::I18nPlugin, inspector::Inspector};

pub struct ExtensionsPlugin;

impl Plugin for ExtensionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorMenuEntrys>();
        app.add_plugins((I18nPlugin, Game, Docking, CommandPanel, Editing, Inspector, ));
    }
}

#[derive(DerefMut, Deref, Resource, Default)]
pub struct EditorMenuEntrys(ItemGroup);

pub trait MenuExt {
    fn menu_context(&mut self, add_menu: impl FnOnce(MenuContext)) -> &mut Self;
}

pub struct MenuContext<'w> {
    item: ItemAsContainer<'w>,
    world: &'w mut World
}

impl MenuContext<'_> {
    pub fn inside_sub<'a>(
        &mut self,
        id: &'a str,
        add_sub: impl FnOnce(MenuContext),
    ) -> Result<(), MenuError<'a>> {
        let item = self
            .item
            .get_item_mut(id)
            .ok_or(MenuError::NotFound { id })?;
        add_sub(MenuContext {
            item: item
                .source
                .as_container()
                .ok_or(MenuError::NotAContainer { id })?,
            world: self.world
        });
        Ok(())
    }
    pub fn with_category(
        &mut self,
        id: &str,
        name: Cow<'static, str>,
        piority: usize,
        add_sub: impl FnOnce(MenuContext),
    ) {
        self.add(id, name, Category::default(), piority);
        self.inside_sub(id, add_sub).unwrap();
    }
    pub fn with_sub_menu(
        &mut self,
        id: &str,
        name: Cow<'static, str>,
        piority: usize,
        add_sub: impl FnOnce(MenuContext),
    ) {
        self.add(id, name, SubMenu::default(), piority);
        self.inside_sub(id, add_sub).unwrap();
    }
    pub fn add(
        &mut self,
        id: &str,
        name: Cow<'static, str>,
        item: impl Into<MenuItemVariant>,
        piority: usize,
    ) {
        let mut source = item.into();
        source.initialize(self.world);
        self.item.add_item(
            id,
            MenuItem {
                name,
                source,
                piority,
            },
        );
    }
}

impl MenuExt for App {
    fn menu_context(&mut self, add_menu: impl FnOnce(MenuContext)) -> &mut Self {
        self.world_mut().resource_scope(|world, mut entrys:bevy::prelude::Mut<'_, EditorMenuEntrys> | {
            let container = entrys.as_container();
            add_menu(MenuContext { item: container, world });
        });
        self
    }
}

#[derive(Debug, Snafu)]
pub enum MenuError<'a> {
    #[snafu(display("Id {id} not found"))]
    NotFound { id: &'a str },
    #[snafu(display("{id} is not a container"))]
    NotAContainer { id: &'a str },
}
