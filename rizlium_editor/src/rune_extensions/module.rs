use rune::{ContextError, Module};

use super::types::{
    ActionRegistration, DelegateUi, HotkeyRegistration, MenuRegistration, RuneRegistrar,
    TabRegistration,
};

/// 创建helium框架的Rune模块
pub fn create_helium_module() -> Result<Module, ContextError> {
    let mut module = Module::with_crate("helium")?;

    // 注册RuneRegistrar类型
    module.ty::<RuneRegistrar>()?;
    module.function_meta(RuneRegistrar::register_tab)?;
    module.function_meta(RuneRegistrar::register_action)?;
    module.function_meta(RuneRegistrar::register_hotkey)?;
    module.function_meta(RuneRegistrar::register_menu)?;

    // 注册DelegateUi类型
    module.ty::<DelegateUi>()?;
    module.function_meta(DelegateUi::label)?;
    module.function_meta(DelegateUi::heading)?;
    module.function_meta(DelegateUi::button)?;
    module.function_meta(DelegateUi::horizontal)?;
    module.function_meta(DelegateUi::vertical)?;
    module.function_meta(DelegateUi::add_space)?;
    module.function_meta(DelegateUi::separator)?;

    // 注册辅助类型
    module.ty::<TabRegistration>()?;
    module.ty::<ActionRegistration>()?;
    module.ty::<HotkeyRegistration>()?;
    module.ty::<MenuRegistration>()?;

    Ok(module)
}
