use std::{path::PathBuf, sync::Arc};

use anyhow::Context;
use bevy::{prelude::*, tasks::block_on};
use helium_framework::prelude::{ActionsExt, MenuRegistration, ToastsStorage};
use rizlium_dyn_plugin::{fetch_and_compile_extensions, load_compiled_extensions, RuneEngine};

// 导入模块
pub mod module;
pub mod types;
pub mod wrappers;

use crate::MainMenuContext;
use types::RuneRegistrar;

/// 扩展现有的Rune上下文以支持helium框架
pub fn extend_rune_context_with_helium(context: &mut rune::Context) -> anyhow::Result<()> {
    let module = module::create_helium_module()?;
    context.install(module)?;
    Ok(())
}

/// 将helium支持集成到现有的加载流程中
pub fn process_rune_extension_with_helium(
    extension: &rizlium_dyn_plugin::Extension,
    engine: &rizlium_dyn_plugin::RuneEngine,
    world: &mut World,
) -> anyhow::Result<()> {
    // 创建注册器
    let mut registrar = RuneRegistrar::new();

    // 执行Rune脚本的main函数，传入注册器
    let mut vm = rune::runtime::Vm::new(engine.runtime_context.clone(), extension.unit().clone());

    vm.call(["main"], (&mut registrar,)).context(
        "Failed to call main function in Rune extension",
    )?;

    // 处理所有注册信息
    wrappers::process_rune_registrations(
        registrar,
        engine.runtime_context.clone(),
        world,
    )?;

    Ok(())
}

/// 为现有的扩展加载器添加helium支持
pub struct HeliumRuneSupportPlugin;

impl Plugin for HeliumRuneSupportPlugin {
    fn build(&self, app: &mut App) {
        // 创建RuneEngine资源, 并加入helium支持
        let mut context = rune::Context::new();
        extend_rune_context_with_helium(&mut context).expect("Failed to extend Rune context");
        let runtime = context.runtime().unwrap();
        app.insert_resource(RuneEngine {
            context,
            runtime_context: Arc::new(runtime),
        });
        // 注册menu: 重载plugins
        app.reflect_system("plugins.reload", "Debug: Reload plugins",reload_plugins)
            .register_command::<MainMenuContext>("reload_plugins", "Reloads all plugins", "plugins.reload");
    }
}

fn reload_plugins(world: &mut World) {
    let result = block_on(fetch_and_compile_extensions(
        PathBuf::from("plugins"),
        world.resource::<RuneEngine>(),
    ));
    let Ok(mut extentions) = result else {
        world
            .resource_mut::<ToastsStorage>()
            .error(format!("Failed to load plugins: {}", result.err().unwrap()));
        return;
    };
    // 更新runtime_context
    let mut engine = world.resource_mut::<RuneEngine>();
    engine.runtime_context = Arc::new(engine.context.runtime().unwrap());
    world.resource_scope(|world, engine: Mut<RuneEngine>| {
        if let Err(e) = block_on(
            load_compiled_extensions(
                &mut extentions,
                &engine,
                Box::new(|ext, engine| {
                    process_rune_extension_with_helium(ext, engine, world)?;
                    Ok(())
                }),
            )
        ) {
            world
                .resource_mut::<ToastsStorage>()
                .error(format!("Failed to load plugins: {e:#}"));
        }
    });
    world.insert_resource(extentions);

}
