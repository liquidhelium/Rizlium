use std::fmt::Debug;

use bevy::prelude::{World, Mut};
use egui::Ui;
use enum_dispatch::enum_dispatch;
use indexmap::IndexMap;

use crate::{hotkeys::Action, ActionId, ActionStorages};

#[enum_dispatch(MenuItemProvider)]
#[derive(Debug, Clone)]
pub enum MenuItemVariant {
    Button,
    SubMenu,
    Category,
}

impl MenuItemVariant {
    pub fn kind_eq(&self, other: &Self) -> bool {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct MenuItem {
    pub name: String,
    pub source: MenuItemVariant,
    pub piority: usize,
}

#[enum_dispatch]
pub trait MenuItemProvider {
    fn ui(&self, ui: &mut Ui, world: &mut World, name: &str);
    fn find_subitem_mut(&mut self, _id: &str) -> Option<&mut MenuItem> {
        None
    }
    fn find_subitem_recursive(&mut self, id: &str) -> Option<&mut MenuItem> {
        let split: Vec<&str> = id.splitn(2, ".").collect();
        let id = *split.get(0)?;
        let item = self.find_subitem_mut(id)?;
        if let Some(trailing) = split.get(1) {
            item.source.find_subitem_recursive(trailing)
        } else {
            Some(item)
        }
    }
    fn as_container(&mut self) -> Option<ItemAsContainer> {
        None
    }
}

pub trait ContainerItem<'item> {
    fn add_item(&mut self, id: &str, menu_item: MenuItem);
    fn remove_item(&mut self, id: &str) -> Option<MenuItem>;
    fn get_item(&self, id: &str) -> Option<&MenuItem>;
    fn get_item_mut(&mut self, id: &str) -> Option<&mut MenuItem>;
}

pub struct ItemAsContainer<'item> {
    container_item: Box<dyn ContainerItem<'item> + 'item>,
}
impl ItemAsContainer<'_> {
    pub fn add_item(&mut self, id: &str, menu_item: MenuItem) {
        self.container_item.add_item(id, menu_item)
    }
    pub fn remove_item(&mut self, id: &str) -> Option<MenuItem> {
        self.container_item.remove_item(id)
    }
    pub fn get_item(&self, id: &str) -> Option<&MenuItem> {
        self.container_item.get_item(id)
    }
    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut MenuItem> {
        self.container_item.get_item_mut(id)
    }
}

#[derive(Clone)]
pub struct Button {
    action: ActionId,
}

impl Button {
    pub fn new(action: ActionId) -> Self {
        Self {
            action,
        }
    }
}

impl Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<Button>")
    }
}

impl MenuItemProvider for Button {
    fn ui(&self, ui: &mut Ui, world: &mut World, name: &str) {
        if ui.button(name).clicked() {
            world.resource_scope(|world: &mut World, mut actions: Mut<ActionStorages>| {
                let _ = actions.run_instant(&self.action, (), world).map_err(|err| bevy::prelude::error!("encountered error when running action: {}", err));
            });
            ui.close_menu();
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ItemGroup {
    items: IndexMap<String, MenuItem>,
}

impl ItemGroup {
    pub fn iter_items(&self) -> impl Iterator<Item = &MenuItem> {
        self.items.values()
    }
    pub fn foreach_ui(&self, ui: &mut Ui, world: &mut World) {
        for item in self.iter_items() {
            item.source.ui(ui, world, &item.name);
        }
    }
    pub fn as_container(&mut self) -> ItemAsContainer {
        ItemAsContainer {
            container_item: Box::new(ItemGroupAsContainer {group: self}),
        }
    }
}

struct ItemGroupAsContainer<'group> {
    group: &'group mut ItemGroup,
}

impl<'item> ContainerItem<'item> for ItemGroupAsContainer<'item> {
    fn add_item(&mut self, id: &str, menu_item: MenuItem) {
        self.group.items.insert(id.to_string(), menu_item);
        self.group
            .items
            .sort_unstable_by(|_, item1, _, item2| item1.piority.cmp(&item2.piority));
    }
    fn get_item_mut(&mut self, id: &str) -> Option<&mut MenuItem> {
        self.group.items.get_mut(id)
    }
    fn get_item(&self, id: &str) -> Option<&MenuItem> {
        self.group.items.get(id)
    }
    fn remove_item(&mut self, id: &str) -> Option<MenuItem> {
        self.group.items.remove(id)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SubMenu {
    group: ItemGroup,
}

#[derive(Debug, Clone, Default)]
pub struct Category {
    group: ItemGroup,
}
impl MenuItemProvider for SubMenu {
    fn ui(&self, ui: &mut Ui, world: &mut World, name: &str) {
        ui.menu_button(name, |ui| self.group.foreach_ui(ui, world));
    }
    fn find_subitem_mut(&mut self, sub_id: &str) -> Option<&mut MenuItem> {
        self.group.items.get_mut(sub_id)
    }
    fn as_container<'a>(&'a mut self) -> Option<ItemAsContainer<'a>> {
        Some(self.group.as_container())
    }
}

impl MenuItemProvider for Category {
    fn ui(&self, ui: &mut Ui, world: &mut World, name: &str) {
        ui.label(name);
        ui.separator();
        self.group.foreach_ui(ui, world);
    }
    fn find_subitem_mut(&mut self, sub_id: &str) -> Option<&mut MenuItem> {
        self.group.items.get_mut(sub_id)
    }
    fn as_container(&mut self) -> Option<ItemAsContainer> {
        Some(self.group.as_container())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_item() {
        let mut menu = dbg!(construct_menu());
        dbg!(menu.source.find_subitem_recursive("item1"));
        dbg!(menu.source.find_subitem_recursive("category2.item1"));
        dbg!(menu
            .source
            .find_subitem_recursive("category2.item1.nonexist"));
    }

    fn construct_menu() -> MenuItem {
        let category = MenuItem {
            name: "category".into(),
            source: Category {
                group: ItemGroup {
                    items: [
                        ("item1".into(), button_with_name("Item1".into())),
                        ("item2".into(), button_with_name("Item2".into())),
                    ]
                    .into_iter()
                    .collect(),
                },
            }
            .into(),
            piority: 0,
        };
        let sub_menu = MenuItem {
            name: "menu".into(),
            source: SubMenu {
                group: ItemGroup {
                    items: [
                        ("item1".into(), button_with_name("Item1".into())),
                        ("category2".into(), category),
                    ]
                    .into_iter()
                    .collect(),
                },
            }
            .into(),
            piority: 1,
        };
        sub_menu
    }

    fn button_with_name(name: String) -> MenuItem {
        MenuItem {
            name,
            source: Button {
                action: "play_genshin".into(),
            }
            .into(),
            piority: 0,
        }
    }
}
