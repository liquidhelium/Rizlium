mod game;
pub mod docking;

use bevy::prelude::{App, Deref, DerefMut, Plugin, Resource};
use snafu::Snafu;

use crate::menu::{ItemAsContainer, MenuItem, MenuItemProvider, MenuItemVariant, SubMenu, Category, ItemGroup};

use self::game::Game;

pub struct ExtensionsPlugin;

impl Plugin for ExtensionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EditorMenuEntrys>();
        app.add_plugins(Game);
    }
}

#[derive(DerefMut, Deref, Resource, Default)]
pub struct EditorMenuEntrys(ItemGroup);

pub trait MenuExt {
    fn menu_context(&mut self, add_menu: impl FnOnce(MenuContext));
}

pub struct MenuContext<'w> {
    item: ItemAsContainer<'w>,
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
        });
        Ok(())
    }
    pub fn with_category<'a>(&mut self,id: &'a str, name: String, piority: usize, add_sub: impl FnOnce(MenuContext)) {
        self.add(id, name, Category::default(), piority);
        self.inside_sub(id, add_sub).unwrap();
    }
    pub fn with_sub_menu<'a>(&mut self,id: &'a str, name: String, piority: usize, add_sub: impl FnOnce(MenuContext)) {
        self.add(id, name, SubMenu::default(), piority);
        self.inside_sub(id, add_sub).unwrap();
    }
    pub fn add(
        &mut self,
        id: &str,
        name: String,
        item: impl Into<MenuItemVariant>,
        piority: usize,
    ) {
        self.item.add_item(
            id,
            MenuItem {
                name,
                source: item.into(),
                piority,
            },
        );
    }
}

impl MenuExt for App {
    fn menu_context(&mut self, add_menu: impl FnOnce(MenuContext)) {
        let mut entrys = self.world.resource_mut::<EditorMenuEntrys>();
        let container = entrys.as_container();
        add_menu(MenuContext { item: container });
    }
}

#[derive(Debug, Snafu)]
pub enum MenuError<'a> {
    #[snafu(display("Id {id} not found"))]
    NotFound { id: &'a str },
    #[snafu(display("{id} is not a container"))]
    NotAContainer { id: &'a str },
}
