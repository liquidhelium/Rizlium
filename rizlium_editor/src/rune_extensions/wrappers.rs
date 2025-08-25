use std::sync::Arc;

use bevy::prelude::*;
use egui::Ui;
use helium_framework::prelude::{ActionsExt, TabRegistrationExt};
use rune::{alloc::clone::TryClone as _, runtime::{Function, RuntimeContext}};

use super::types::{DelegateUi, RuneRegistrar};

/// 将收集的注册信息转换为实际的Bevy系统并注册到helium框架
pub fn process_rune_registrations(
    registrar: RuneRegistrar,
    context: Arc<RuntimeContext>,
    world: &mut World,
) -> anyhow::Result<()>{
    // 处理tab注册
    for tab in registrar.tabs {
        let func = tab.func.try_clone()?.into_sync()?;
        
        let system = move |InMut(ui): InMut<'_, Ui>| {
            let delegate_ui = DelegateUi::new(ui);
            let _ = func.call::<(DelegateUi,)>((delegate_ui,));
        };
        
        // 注册到helium tab系统
        world.register_tab(
            tab.id.as_str(),
            tab.title,
            system,
            move || true, // 这里可以根据需要实现更复杂的条件
        );
    }
    
    // 处理action注册
    for action in registrar.actions {
        let func = action.func.try_clone()?.into_sync()?;
        let system = move || {
            let _ = func.call::<()>(());
        };
        
        // 注册到helium action系统
        world.reflect_system(action.id.as_str(), &action.description, system);
    }
    
    // 处理hotkey注册
    for hotkey in registrar.hotkeys {
        let func = hotkey.func.try_clone()?.into_sync()?;
        let system = move || {
            let _ = func.call::<()>(());
        };
        
        let system_id = world.register_system(system);
        
        // 注册到helium action系统
        world.commands().queue(move |world: &mut World| {
            world.resource_scope(|world, mut registry: Mut<'_, helium_framework::reflect_system::RSystemRegistry>| {
                let meta = helium_framework::reflect_system::ReflectSystemMeta {
                    id: hotkey.id.clone().as_str().into(),
                    description: hotkey.description.clone(),
                    system_id: helium_framework::reflect_system::ReflectSystemId::from_system_id(system_id),
                    input: "()".to_string(),
                    output: "()".to_string(),
                };
            });
        });
    }
    
    // 处理menu注册
    for menu in registrar.menus {
        // 注册到helium menu系统
        world.resource_scope(|world, mut menu_system: Mut<'_, helium_framework::menu_system::MenuSystem>| {
            menu_system.register(
                helium_framework::menu_system::MenuItem::new(
                    menu.title.clone(),
                    menu.path.clone(),
                    helium_framework::menu_system::Action::Command(menu.action_id.as_str().into(), std::marker::PhantomData::<()>)
                ),
                world
            );
        });
    }
    Ok(())
}