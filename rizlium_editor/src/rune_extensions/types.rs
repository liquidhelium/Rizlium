use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use egui::{Ui, UiBuilder};
use rune::{Any, runtime::Function};

/// Rune可访问的UI包装器
#[derive(Any, Clone)]
pub struct DelegateUi {
    pub ui: Arc<Mutex<Ui>>,
}

impl DelegateUi {
    pub fn new(ui: &mut Ui) -> Self {
        Self {
            ui: Arc::new(Mutex::new(ui.new_child(UiBuilder::new()))),
        }
    }

    #[rune::function]
    pub fn label(&mut self, text: &str) {
        let mut ui = self.ui.lock().unwrap();
        ui.label(text);
    }

    #[rune::function]
    pub fn heading(&mut self, text: &str) {
        let mut ui = self.ui.lock().unwrap();
        ui.heading(text);
    }

    #[rune::function]
    pub fn button(&mut self, text: &str) -> bool {
        let mut ui = self.ui.lock().unwrap();
        ui.button(text).clicked()
    }

    #[rune::function]
    pub fn horizontal(&mut self, f: Function) {
        let mut ui = self.ui.lock().unwrap();
        ui.horizontal(|ui| {
            let delegate = DelegateUi::new(ui);
            let _ = f.call::<(DelegateUi,)>((delegate,));
        });
    }

    #[rune::function]
    pub fn vertical(&mut self, f: Function) {
        let mut ui = self.ui.lock().unwrap();
        ui.vertical(|ui| {
            let delegate = DelegateUi::new(ui);
            let _ = f.call::<(DelegateUi,)>((delegate,));
        });
    }

    #[rune::function]
    pub fn add_space(&mut self, space: f32) {
        let mut ui = self.ui.lock().unwrap();
        ui.add_space(space);
    }

    #[rune::function]
    pub fn separator(&mut self) {
        let mut ui = self.ui.lock().unwrap();
        ui.separator();
    }
}

/// Tab注册信息
#[derive(Any)]
pub struct TabRegistration {
    pub id: String,
    pub title: String,
    pub func: Function,
}

/// Action注册信息
#[derive(Any)]
pub struct ActionRegistration {
    pub id: String,
    pub description: String,
    pub func: Function,
}

/// Hotkey注册信息
#[derive(Any)]
pub struct HotkeyRegistration {
    pub id: String,
    pub keys: Vec<String>,
    pub description: String,
    pub func: Function,
}

/// Menu注册信息
#[derive(Any)]
pub struct MenuRegistration {
    pub path: String,
    pub title: String,
    pub action_id: String,
}

/// Rune注册器 - 收集所有注册信息
#[derive(Any)]
pub struct RuneRegistrar {
    pub tabs: Vec<TabRegistration>,
    pub actions: Vec<ActionRegistration>,
    pub hotkeys: Vec<HotkeyRegistration>,
    pub menus: Vec<MenuRegistration>,
}

impl RuneRegistrar {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            actions: Vec::new(),
            hotkeys: Vec::new(),
            menus: Vec::new(),
        }
    }

    #[rune::function]
    pub fn register_tab(&mut self, id: &str, title: &str, func: Function) {
        self.tabs.push(TabRegistration {
            id: id.to_string(),
            title: title.to_string(),
            func,
        });
    }

    #[rune::function]
    pub fn register_action(&mut self, id: &str, description: &str, func: Function) {
        self.actions.push(ActionRegistration {
            id: id.to_string(),
            description: description.to_string(),
            func,
        });
    }

    #[rune::function]
    pub fn register_hotkey(&mut self, id: &str, keys: Vec<String>, description: &str, func: Function) {
        self.hotkeys.push(HotkeyRegistration {
            id: id.to_string(),
            keys,
            description: description.to_string(),
            func,
        });
    }

    #[rune::function]
    pub fn register_menu(&mut self, path: &str, title: &str, action_id: &str) {
        self.menus.push(MenuRegistration {
            path: path.to_string(),
            title: title.to_string(),
            action_id: action_id.to_string(),
        });
    }
}