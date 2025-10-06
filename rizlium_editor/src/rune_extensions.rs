// 引入标准库中的路径处理模块和原子引用计数智能指针。
use std::{path::PathBuf, sync::Arc};

// 引入 `anyhow` 库，用于提供更具上下文的错误处理。
use anyhow::Context;
// 引入 Bevy 引擎的核心预设模块和任务阻塞执行功能。
use bevy::{prelude::*, tasks::block_on};
// 引入自定义的 `helium_framework` 中的核心 trait 和资源。
use helium_framework::prelude::{ActionsExt, MenuRegistration, ToastsStorage};
// 引入动态插件加载库 `rizlium_dyn_plugin` 的核心功能。
use rizlium_dyn_plugin::{fetch_and_compile_extensions, load_compiled_extensions, RuneEngine};

// 声明并公开当前 crate 下的子模块。
pub mod module; // 定义了暴露给 Rune 脚本的 `helium` 模块。
pub mod types; // 定义了 Rune 与 Bevy 交互时使用的数据结构，如 `RuneRegistrar`。
pub mod wrappers; // 包含了处理从 Rune 到 Bevy 的注册逻辑的包装函数。

// 引入主菜单上下文，用于向主菜单注册命令。
use crate::MainMenuContext;
// 引入 `RuneRegistrar`，这是 Rune 脚本用来向 Bevy 应用注册各种功能的中间结构。
use types::RuneRegistrar;

/// 扩展现有的 Rune 上下文（`rune::Context`），为其安装 `helium` 模块。
/// `helium` 模块暴露了框架的核心功能（如注册系统、菜单项等）给 Rune 脚本。
///
/// # 参数
/// * `context`: 一个可变的 `rune::Context` 引用，将向其中安装新模块。
///
/// # 返回值
/// * `anyhow::Result<()>`: 如果模块创建和安装成功，则返回 `Ok(())`，否则返回错误。
pub fn extend_rune_context_with_helium(context: &mut rune::Context) -> anyhow::Result<()> {
    // 调用 `module::create_helium_module` 函数来构建 `helium` 模块。
    let module = module::create_helium_module()?;
    // 使用 `context.install` 方法将模块安装到 Rune 的上下文中。
    context.install(module)?;
    // 返回成功。
    Ok(())
}

/// 处理单个已加载的 Rune 扩展，执行其 `main` 函数并处理其注册请求。
/// 这是 `rizlium_dyn_plugin` 加载流程的回调函数，用于集成 `helium_framework` 的特定逻辑。
///
/// # 参数
/// * `extension`: 对已加载的 Rune 扩展的引用。
/// * `engine`: 对 `RuneEngine` 资源的引用，包含了 Rune 的运行时上下文。
/// * `world`: 对 Bevy `World` 的可变引用，用于注册系统、资源等。
pub fn process_rune_extension_with_helium(
    extension: &rizlium_dyn_plugin::Extension,
    engine: &rizlium_dyn_plugin::RuneEngine,
    world: &mut World,
) -> anyhow::Result<()> {
    // 创建一个新的 `RuneRegistrar` 实例，它将被传递给 Rune 脚本。
    let mut registrar = RuneRegistrar::new();

    // 创建一个 Rune 虚拟机（VM）实例，用于执行脚本代码。
    let mut vm = rune::runtime::Vm::new(engine.runtime_context.clone(), extension.unit().clone());

    // 调用 Rune 脚本中名为 `main` 的函数，并将 `registrar` 的可变引用作为参数传入。
    // `context` 方法用于在发生错误时附加一条描述性的错误信息。
    vm.call(["main"], (&mut registrar,))
        .context("Failed to call main function in Rune extension")?;

    // 在 Rune 脚本执行完毕后，处理 `registrar` 中收集到的所有注册请求。
    wrappers::process_rune_registrations(
        registrar,                      // 包含了所有来自 Rune 脚本的注册信息。
        engine.runtime_context.clone(), // Rune 的运行时上下文，用于后续的函数调用。
        world,                          // Bevy 的 `World`，用于实际执行注册操作。
    )?;

    // 返回成功。
    Ok(())
}

/// `HeliumRuneSupportPlugin` 是一个 Bevy 插件，用于设置和集成 Rune 脚本支持。
pub struct HeliumRuneSupportPlugin;

impl Plugin for HeliumRuneSupportPlugin {
    fn build(&self, app: &mut App) {
        // --- 1. 创建并配置 RuneEngine 资源 ---
        // 创建一个新的 Rune 上下文。
        let mut context = rune::Context::new();
        // 使用我们定义的函数来扩展此上下文，为其添加 `helium` 模块。
        extend_rune_context_with_helium(&mut context).expect("Failed to extend Rune context");
        // 从上下文中构建运行时（Runtime Context），这是一个轻量级的、可共享的上下文版本。
        let runtime = context.runtime().unwrap();
        // 将 `RuneEngine`（包含完整的上下文和共享的运行时上下文）作为资源插入到 Bevy 应用中。
        app.insert_resource(RuneEngine {
            context,
            runtime_context: Arc::new(runtime),
        });

        // --- 2. 注册“重载插件”的命令和菜单项 ---
        // `reflect_system` 将一个普通的 Rust 函数（`reload_plugins`）注册为一个“反射系统”，
        // 使其可以通过字符串 ID 被调用。
        app.reflect_system("plugins.reload", "Debug: Reload plugins", reload_plugins)
            // `register_command` 将这个反射系统与一个菜单项关联起来。
            // 当用户点击 "Debug" -> "Reload plugins" 菜单时，就会执行 `reload_plugins` 系统。
            .register_command::<MainMenuContext>(
                "reload_plugins",
                "Reloads all plugins",
                "plugins.reload",
            );
    }
}

/// `reload_plugins` 是一个 Bevy 系统，用于重新编译和加载所有位于 `plugins/` 目录下的 Rune 脚本。
fn reload_plugins(world: &mut World) {
    // `block_on` 会阻塞当前线程，直到异步操作完成。
    // 这里我们异步地查找、编译所有插件。
    let result = block_on(fetch_and_compile_extensions(
        PathBuf::from("plugins"),       // 插件目录。
        world.resource::<RuneEngine>(), // 传入 Rune 引擎用于编译。
    ));
    // 检查编译结果。
    let Ok(mut extentions) = result else {
        // 如果编译失败，则显示一个错误通知。
        world
            .resource_mut::<ToastsStorage>()
            .error(format!("Failed to load plugins: {}", result.err().unwrap()));
        return;
    };
    // --- 更新 Rune 引擎的运行时上下文 ---
    // 因为可能添加了新的类型或函数，需要重新生成运行时上下文。
    let mut engine = world.resource_mut::<RuneEngine>();
    engine.runtime_context = Arc::new(engine.context.runtime().unwrap());

    // 使用 `resource_scope` 来安全地借用 `RuneEngine` 资源。
    world.resource_scope(|world, engine: Mut<RuneEngine>| {
        // 异步加载所有已成功编译的扩展。
        if let Err(e) = block_on(load_compiled_extensions(
            &mut extentions,
            &engine,
            // 传入一个回调闭包，用于处理每个加载的扩展。
            Box::new(|ext, engine| {
                // 在这个闭包中，我们调用之前定义的 `process_rune_extension_with_helium` 函数，
                // 来执行脚本的 `main` 函数并处理其注册。
                process_rune_extension_with_helium(ext, engine, world)?;
                Ok(())
            }),
        )) {
            // 如果加载过程中（即执行 `main` 函数时）出错，则显示错误通知。
            world
                .resource_mut::<ToastsStorage>()
                .error(format!("Failed to load plugins: {e:#}"));
        }
    });
    // 将新加载的扩展集合作为资源插入到 `World` 中，替换掉旧的。
    world.insert_resource(extentions);
}
