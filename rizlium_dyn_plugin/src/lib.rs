//! usage:
//! 1. fetch and compile extensions from a directory
//! 2. makeup your own handler
//! 3. load the compiled extensions

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use async_fs::read_to_string;
use bevy::{
    prelude::*,
    tasks::{Task, futures_lite::StreamExt as _},
};
use rune::{
    Context, Diagnostics, FromValue, Sources, Unit,
    runtime::{Function, RuntimeContext},
};

pub struct Extension {
    sources: Sources,
    unit: Arc<Unit>,
}

impl Extension {
    pub fn sources(&self) -> &Sources {
        &self.sources
    }
}

pub enum ExtensionHandle {
    Loading(Task<Extension>),
    Compiled(Extension),
    Loaded(Extension),
}

#[derive(Resource)]
pub struct ExtensionsStorage {
    extensions: HashMap<String, ExtensionHandle>,
}

#[derive(Resource)]
pub struct RuneEngine {
    context: Context,
    runtime_context: Arc<RuntimeContext>,
}

pub fn compile_extension(string: String, context: &Context) -> anyhow::Result<Extension> {
    // we only put one source inside a `Sources`, so the name is hard-coded.
    let source = rune::Source::new("entry", string)?;
    let mut sources = Sources::new();
    let mut diagnostics = Diagnostics::new();
    sources.insert(source)?;
    let unit = rune::prepare(&mut sources)
        .with_context(context)
        .with_diagnostics(&mut diagnostics)
        .build()?;

    Ok(Extension {
        sources,
        unit: Arc::new(unit),
    })
}

pub async fn fetch_and_compile_extensions(
    path: PathBuf,
    engine: &RuneEngine,
) -> anyhow::Result<ExtensionsStorage> {
    // get the scripts in the directory
    let mut entries = async_fs::read_dir(path).await?;
    let mut extensions = HashMap::new();
    while let Some(entry) = entries.next().await {
        let entry = entry?;
        // check if the entry is a file and has the ".rn" extension
        if entry.file_type().await?.is_file() {
            let path = entry.path();
            if let Some(ext) = path.extension()
                && ext == "rn"
            {
                let content = read_to_string(&path).await?;
                extensions.insert(
                    entry.path().to_string_lossy().into_owned(),
                    ExtensionHandle::Compiled(compile_extension(content, &engine.context)?),
                );
            }
        }
    }
    Ok(ExtensionsStorage { extensions })
}

pub struct ExtensionFunctions {
    pub hotkey: Function,
    pub tab: Function,
    pub menu: Function,
    pub actions: Function,
}

pub async fn load_compiled_extensions(
    storage: &mut ExtensionsStorage,
    engine: &RuneEngine,
    handler: &mut dyn FnMut(ExtensionFunctions) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
    for (name, handle) in storage.extensions.iter_mut() {
        if let ExtensionHandle::Compiled(extension) = handle {
            let unit = extension.unit.clone();
            let mut vm = rune::runtime::Vm::new(engine.runtime_context.clone(), unit.clone());
            let rune::runtime::VmResult::Ok(result) =
                vm.execute(["main"], ())?.async_complete().await
            else {
                return Err(anyhow::anyhow!("Failed to execute extension: {}", name));
            };
            let mut result: rune::alloc::HashMap<rune::alloc::String, Function> =
                rune::alloc::HashMap::from_value(result)?;

            let (Some(hotkey), Some(tab), Some(menu), Some(actions)) = (
                result.remove("hotkey"),
                result.remove("tab"),
                result.remove("menu"),
                result.remove("actions"),
            ) else {
                return Err(anyhow::anyhow!(
                    "Extension {} is missing required functions",
                    name
                ));
            };
            let functions = ExtensionFunctions {
                hotkey,
                tab,
                menu,
                actions,
            };
            handler(functions)?;

            // *handle = ExtensionHandle::Loaded(extension);
        }
    }
    Ok(())
}
