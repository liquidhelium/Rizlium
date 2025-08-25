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
    runtime::{Function, RuntimeContext}, termcolor::{ColorChoice, StandardStream}, Context, Diagnostics, FromValue, Sources, Unit
};

pub struct Extension {
    sources: Sources,
    unit: Arc<Unit>,
}

impl Extension {
    pub fn sources(&self) -> &Sources {
        &self.sources
    }
    
    pub fn unit(&self) -> &Arc<Unit>{
        &self.unit
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
    pub context: Context,
    pub runtime_context: Arc<RuntimeContext>,
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
    if !diagnostics.is_empty() {
            let mut writer = StandardStream::stderr(ColorChoice::Always);
            if let Err(e) = diagnostics.emit(&mut writer, &sources) {
                error!("Failed to emit diagnostics: {e}");
            }
        }
    
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
                info!("Compiling extension: {}", path.display());
                info!("Content: {}", content);
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

pub async fn load_compiled_extensions<'c>(
    storage: &mut ExtensionsStorage,
    engine: &RuneEngine,
    mut handler: Box<dyn FnMut(&Extension, &RuneEngine) -> anyhow::Result<()> + 'c>,
) -> anyhow::Result<()> {
    for (_, handle) in storage.extensions.iter_mut() {
        if let ExtensionHandle::Compiled(extension) = handle {
            handler(extension, engine)?;

            // *handle = ExtensionHandle::Loaded(extension);
        }
    }
    Ok(())
}
