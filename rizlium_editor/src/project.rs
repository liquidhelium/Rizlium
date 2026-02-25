use crate::time_and_audio::GameAudioSource;
use crate::extensions::recent::RecentFiles;
#[allow(unused_imports)]
use crate::t;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use bevy_kira_audio::{AudioSource, prelude::StaticSoundData};
use bevy_persistent::Persistent;
use futures_lite::io::AsyncWriteExt;
use helium_framework::prelude::{ActionsExt, ToastsStorage};
use rizlium_chart::parse::rizline::{RizlineChart, RizlineChartMeta};
use rizlium_chart::prelude::*;
use rizlium_render::ChartProvider;
use serde::{Deserialize, Serialize};
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;
pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectState>()
            .init_resource::<RecentFiles>()
            .init_resource::<AsyncTaskRunner>()
            .add_message::<LoadChartEvent>()
            .add_message::<ChartLoadingEvent>()
            .add_message::<SaveChartEvent>()
            .add_systems(
                PostUpdate,
                (
                    handle_load_chart_events,
                    handle_save_chart_events,
                    handle_async_tasks,
                    process_loading_results,
                ),
            );
        app.reflect_system("project.load_path", "Load a path", load_path_action);
        app.reflect_system("project.load_bundle", "Load a bundle", load_bundle_action);
        app.reflect_system(
            "project.save_chart",
            "Save current chart to file",
            save_chart_action,
        );
        app.reflect_system(
            "project.save_chart_as",
            "Save current chart to a new file",
            save_chart_as_action,
        );
        app.reflect_system(
            "project.open_bundle_dialog",
            "Open a chart bundle file",
            open_bundle_dialog_action,
        );
        app.reflect_system(
            "project.open_path_dialog",
            "Open a chart folder",
            open_path_dialog_action,
        );
    }
}

fn load_path_action(In(path): In<String>, mut events: MessageWriter<LoadChartEvent>) {
    events.write(LoadChartEvent::Path(path));
}
fn load_bundle_action(In(path): In<String>, mut events: MessageWriter<LoadChartEvent>) {
    events.write(LoadChartEvent::Bundle(path));
}

fn save_chart_action(mut events: MessageWriter<SaveChartEvent>) {
    events.write(SaveChartEvent);
}

fn save_chart_as_action(mut state: ResMut<ProjectState>, mut runner: ResMut<AsyncTaskRunner>) {
    state.open_save_dialog(&mut runner);
}

fn open_bundle_dialog_action(mut state: ResMut<ProjectState>, mut runner: ResMut<AsyncTaskRunner>) {
    state.open_bundle_dialog(&mut runner);
}

fn open_path_dialog_action(mut state: ResMut<ProjectState>, mut runner: ResMut<AsyncTaskRunner>) {
    state.open_path_dialog(&mut runner);
}

#[derive(Resource, Default)]
pub struct AsyncTaskRunner {
    pub task: Option<RunningTask>,
}

pub enum RunningTask {
    PickFileOpen(Task<Option<PathBuf>>),
    PickBundleOpen(Task<Option<PathBuf>>),
    PickFileSave(Task<Option<PathBuf>>),
    Loading(Task<Result<LoadedChart, ChartLoadingError>>),
    Saving(Task<Result<(), Box<dyn std::error::Error + Send + Sync>>>),
}

#[derive(Resource, Default)]
pub enum ProjectState {
    #[default]
    Idle,
    Loaded(LoadedProject),
}
#[derive(Clone)]
pub struct LoadedChart {
    pub source: SourceKind,
    pub chart: Chart,
    pub audio_source: AudioSource,
    pub path: String,
    pub info: ChartInfo,
}
#[derive(Clone)]
pub enum SourceKind {
    Folder,
    Bundle,
}
#[derive(Debug, Clone)]
pub enum LoadedProject {
    Folder(PathBuf, Chart, ChartInfo),
    Bundle(PathBuf, Chart, ChartInfo),
}
#[derive(Message, Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum LoadChartEvent {
    Bundle(String),
    Path(String),
}

impl LoadChartEvent {
    pub fn text(&self) -> &str {
        use LoadChartEvent::*;
        match self {
            Bundle(s) | Path(s) => s
        }
    }
}

#[derive(Message)]
pub enum ChartLoadingEvent {
    Success(LoadChartEvent),
    Error(ChartLoadingError),
}
#[derive(Message)]
pub struct SaveChartEvent;
#[derive(Debug, snafu::Snafu)]
pub enum ChartLoadingError {
    #[snafu(display("No info file present"))]
    NoInfo { path: String },
    #[snafu(display("Failed to unzip file, source: {source:#?}"), context(false))]
    UnzipFileFailed { source: zip::result::ZipError },
    #[snafu(display("Failed to read file, source: {source:#?}"), context(false))]
    ReadingFileFailed { source: std::io::Error },
    #[snafu(
        display("Chart format is invalid, source: {source:#?}"),
        context(false)
    )]
    ChartFormatInvalid { source: serde_json::Error },
    #[snafu(
        display("Chart info format is invalid, source: {source:#?}"),
        context(false)
    )]
    InfoFormatInvalid { source: serde_yaml::Error },
    #[snafu(
        display("Failed to convert chart, source: {source:#?}"),
        context(false)
    )]
    ChartConvertingFailed {
        source: rizlium_chart::parse::ConvertError,
    },
    #[snafu(
        display("Failed to convert music, source: {source:#?}"),
        context(false)
    )]
    MusicConvertingFailed {
        source: bevy_kira_audio::prelude::FromFileError,
    },
}
#[derive(Deserialize, Clone, Debug)]
pub struct ChartInfo {
    pub name: String,
    pub format: ChartFormat,
    pub chart_path: String,
    pub music_path: String,
}
#[derive(Deserialize, Clone, Debug)]
pub enum ChartFormat {
    Rizline,
    Rizlium,
    RizliumData,
}
impl ChartProvider for ProjectState {
    fn chart(&self) -> &Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::Loaded(project) => match project {
                LoadedProject::Folder(_, chart, _) => chart,
                LoadedProject::Bundle(_, chart, _) => chart,
            },
        }
    }
    fn chart_mut(&mut self) -> &mut Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::Loaded(project) => match project {
                LoadedProject::Folder(_, chart, _) => chart,
                LoadedProject::Bundle(_, chart, _) => chart,
            },
        }
    }
    fn has_chart_system() -> impl SystemCondition<()> {
        IntoSystem::into_system(|state: Res<ProjectState>| {
            matches!(*state, ProjectState::Loaded(_))
        })
    }
}

impl ProjectState {
    pub fn segment_count(&self) -> usize {
        if let Self::Loaded(project) = self {
            let chart = match project {
                LoadedProject::Folder(_, chart, _) => chart,
                LoadedProject::Bundle(_, chart, _) => chart,
            };
            chart
                .lines
                .iter()
                .map(|line| line.points.len().saturating_sub(1))
                .sum()
        } else {
            0
        }
    }
    pub fn note_count(&self) -> usize {
        if let Self::Loaded(project) = self {
            let chart = match project {
                LoadedProject::Folder(_, chart, _) => chart,
                LoadedProject::Bundle(_, chart, _) => chart,
            };
            chart.lines.iter().map(|line| line.notes.len()).sum()
        } else {
            0
        }
    }
    pub fn has_chart(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }
    pub fn loaded_project(&self) -> Option<&LoadedProject> {
        match self {
            Self::Loaded(project) => Some(project),
            _ => None,
        }
    }
}

impl AsyncTaskRunner {
    pub fn is_busy(&self) -> bool {
        self.task.is_some()
    }
    pub fn is_loading(&self) -> bool {
        matches!(self.task, Some(RunningTask::Loading(_)))
    }
}

fn handle_load_chart_events(
    mut events: MessageReader<LoadChartEvent>,
    mut runner: ResMut<AsyncTaskRunner>,
) {
    if events.is_empty() {
        return;
    }
    if let Some(event) = events.read().last() {
        info!("attempting to load chart {event:?}");
        let task = match event {
            LoadChartEvent::Bundle(path) => {
                let path = path.clone();
                IoTaskPool::get().spawn(async move { load_chart_from_bundle(&path).await })
            }
            LoadChartEvent::Path(path) => {
                let path = path.clone();
                IoTaskPool::get().spawn(async move { load_chart_from_path(&path).await })
            }
        };
        runner.task = Some(RunningTask::Loading(task));
    }
    events.clear();
}

fn handle_async_tasks(
    mut runner: ResMut<AsyncTaskRunner>,
    mut state: ResMut<ProjectState>,
    mut events: MessageWriter<LoadChartEvent>,
    mut loading_events: MessageWriter<ChartLoadingEvent>,
    mut toasts: ResMut<ToastsStorage>,
    asset_server: Res<AssetServer>,
    mut command: Commands,
) {
    let Some(task_enum) = runner.task.as_mut() else {
        return;
    };

    match task_enum {
        RunningTask::PickFileOpen(task) => {
            if let Some(result) =
                futures_lite::future::block_on(futures_lite::future::poll_once(task))
            {
                if let Some(path) = result {
                    events.write(LoadChartEvent::Path(path.to_string_lossy().into_owned()));
                }
                runner.task = None;
            }
        }
        RunningTask::PickBundleOpen(task) => {
            if let Some(result) =
                futures_lite::future::block_on(futures_lite::future::poll_once(task))
            {
                if let Some(path) = result {
                    events.write(LoadChartEvent::Bundle(path.to_string_lossy().into_owned()));
                }
                runner.task = None;
            }
        }
        RunningTask::PickFileSave(task) => {
            if let Some(result) =
                futures_lite::future::block_on(futures_lite::future::poll_once(task))
            {
                if let Some(path) = result {
                    // Determine format from extension (if not loaded)
                    let format = if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                        match ext {
                            "json" => ChartFormat::Rizline,
                            "rzd" => ChartFormat::RizliumData,
                            _ => ChartFormat::Rizlium,
                        }
                    } else {
                        ChartFormat::Rizlium
                    };

                    if let ProjectState::Loaded(project) = &*state {
                        let chart = match project {
                            LoadedProject::Folder(_, chart, _) => chart.clone(),
                            LoadedProject::Bundle(_, chart, _) => chart.clone(),
                        };
                        let save_task = IoTaskPool::get()
                            .spawn(async move { save_chart_to_file(&chart, &path, format).await });
                        runner.task = Some(RunningTask::Saving(save_task));
                        return; // Don't clear task yet, we replaced it
                    }
                }
                runner.task = None;
            }
        }
        RunningTask::Loading(task) => {
            if let Some(result) =
                futures_lite::future::block_on(futures_lite::future::poll_once(task))
            {
                match result {
                    Ok(loaded) => {
                        let path = loaded.path.clone();
                        match loaded.source {
                            SourceKind::Folder => {
                                *state = ProjectState::Loaded(LoadedProject::Folder(
                                    PathBuf::from(path),
                                    loaded.chart,
                                    loaded.info,
                                ));
                            }
                            SourceKind::Bundle => {
                                *state = ProjectState::Loaded(LoadedProject::Bundle(
                                    PathBuf::from(path),
                                    loaded.chart,
                                    loaded.info,
                                ));
                            }
                        }
                        let handle = asset_server.add(loaded.audio_source);
                        command.insert_resource(GameAudioSource(handle));
                        let event = match loaded.source {
                            SourceKind::Bundle => LoadChartEvent::Bundle,
                            SourceKind::Folder => LoadChartEvent::Path,
                        };
                        loading_events.write(ChartLoadingEvent::Success(event(loaded.path)));
                    }
                    Err(err) => {
                        *state = ProjectState::Idle;
                        loading_events.write(ChartLoadingEvent::Error(err));
                    }
                }
                runner.task = None;
            }
        }
        RunningTask::Saving(task) => {
            if let Some(result) =
                futures_lite::future::block_on(futures_lite::future::poll_once(task))
            {
                match result {
                    Ok(_) => {
                        toasts.info(t!("project-save-success"));
                    }
                    Err(e) => {
                        error!("Failed to save chart: {e}");
                        toasts.error(t!("project-save-fail", "err" => e.to_string()));
                    }
                }
                runner.task = None;
            }
        }
    }
}

fn handle_save_chart_events(
    mut events: MessageReader<SaveChartEvent>,
    state: Res<ProjectState>,
    mut runner: ResMut<AsyncTaskRunner>,
) {
    for _ in events.read() {
        if let ProjectState::Loaded(project) = &*state {
            let (chart, path, format) = match project {
                LoadedProject::Folder(p, chart, info) => {
                    (chart.clone(), p.join(&info.chart_path), info.format.clone())
                }
                LoadedProject::Bundle(p, chart, info) => {
                    (chart.clone(), p.with_extension("rzl"), info.format.clone())
                }
            };
            let task = IoTaskPool::get()
                .spawn(async move { save_chart_to_file(&chart, &path, format).await });
            runner.task = Some(RunningTask::Saving(task));
        }
    }
}
fn process_loading_results(
    mut events: MessageReader<ChartLoadingEvent>,
    mut recent: ResMut<Persistent<RecentFiles>>,
    mut toast: ResMut<ToastsStorage>,
) {
    for event in events.read() {
        match event {
            ChartLoadingEvent::Success(path) => {
                recent.push(path.clone());
            }
            ChartLoadingEvent::Error(e) => {
                error!("Failed to load chart: {e}");
                toast.error(t!("path-load-fail", "err" => e.to_string()));
            }
        }
    }
}
async fn load_chart_from_bundle(path: &str) -> Result<LoadedChart, ChartLoadingError> {
    let file = async_fs::read(path).await?;
    let mut archive = ZipArchive::new(Cursor::new(&file))?;
    let info_file = archive.by_name("info.yml")?;
    let info: ChartInfo = serde_yaml::from_reader(info_file)?;
    let chart = match info.format {
        ChartFormat::Rizline => {
            let rzl_chart: RizlineChart =
                serde_json::from_reader(archive.by_name(&info.chart_path)?)?;
            rzl_chart
                .try_into()
                .map_err(|e| ChartLoadingError::ChartConvertingFailed { source: e })?
        }
        ChartFormat::Rizlium => serde_json::from_reader(archive.by_name(&info.chart_path)?)?,
        ChartFormat::RizliumData => {
            let data: rizlium_chart::data::Chart =
                serde_json::from_reader(archive.by_name(&info.chart_path)?)?;
            data.into()
        }
    };
    let mut audio_data = Vec::new();
    archive
        .by_name(&info.music_path)?
        .read_to_end(&mut audio_data)?;
    let audio_source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(audio_data))
            .map_err(|e| ChartLoadingError::MusicConvertingFailed { source: e })?,
    };
    Ok(LoadedChart {
        source: SourceKind::Bundle,
        chart,
        audio_source,
        path: path.to_string(),
        info,
    })
}
async fn load_chart_from_path(path: &str) -> Result<LoadedChart, ChartLoadingError> {
    let path_buf = PathBuf::from(path);
    let info_file = async_fs::read(path_buf.join("info.yml"))
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                ChartLoadingError::NoInfo {
                    path: path.to_string(),
                }
            } else {
                ChartLoadingError::ReadingFileFailed { source: e }
            }
        });
    let info: ChartInfo = match info_file {
        Ok(info_file) => serde_yaml::from_reader(Cursor::new(info_file))?,
        Err(_) => match async_fs::read(path_buf.join("metadata.json"))
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    ChartLoadingError::NoInfo {
                        path: path.to_string(),
                    }
                } else {
                    ChartLoadingError::ReadingFileFailed { source: e }
                }
            }) {
            Ok(info) => {
                let meta: RizlineChartMeta = serde_json::from_reader(Cursor::new(info))?;
                ChartInfo {
                    name: meta.title,
                    format: ChartFormat::Rizline,
                    chart_path: "chart.json".into(),
                    music_path: "song.mp3".into(),
                }
            }
            Err(e) => return Err(e),
        },
    };
    let chart = match info.format {
        ChartFormat::Rizline => {
            let chart_file = async_fs::read(path_buf.join(&info.chart_path)).await?;
            let rzl_chart: RizlineChart = serde_json::from_reader(Cursor::new(chart_file))?;
            rzl_chart
                .try_into()
                .map_err(|e| ChartLoadingError::ChartConvertingFailed { source: e })?
        }
        ChartFormat::Rizlium => {
            let chart_file = async_fs::read(path_buf.join(&info.chart_path)).await?;
            serde_json::from_reader(Cursor::new(chart_file))?
        }
        ChartFormat::RizliumData => {
            let chart_file = async_fs::read(path_buf.join(&info.chart_path)).await?;
            let data: rizlium_chart::data::Chart =
                serde_json::from_reader(Cursor::new(chart_file))?;
            data.into()
        }
    };
    let audio_data = async_fs::read(path_buf.join(&info.music_path)).await?;
    let audio_source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(audio_data))
            .map_err(|e| ChartLoadingError::MusicConvertingFailed { source: e })?,
    };
    Ok(LoadedChart {
        source: SourceKind::Folder,
        chart,
        audio_source,
        path: path.to_string(),
        info,
    })
}
async fn save_chart_to_file(
    chart: &Chart,
    path: &Path,
    format: ChartFormat,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let serialized = match format {
        ChartFormat::Rizlium => serde_json::to_vec_pretty(chart)?,
        ChartFormat::RizliumData => {
            let data: rizlium_chart::data::Chart = chart.clone().into();
            serde_json::to_vec_pretty(&data)?
        }
        ChartFormat::Rizline => {
            let rzl_chart: RizlineChart = chart.clone().try_into()?;
            serde_json::to_vec_pretty(&rzl_chart)?
        }
    };
    let mut file = async_fs::File::create(path).await?;
    file.write_all(&serialized).await?;
    file.close().await?;
    Ok(())
}

impl ProjectState {
    pub fn open_bundle_dialog(&mut self, runner: &mut AsyncTaskRunner) {
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog;
            let file = AsyncFileDialog::new()
                .add_filter("Chart Bundle", &["zip"])
                .pick_file()
                .await;
            file.map(|f| f.path().to_string_lossy().into_owned().into())
        });
        runner.task = Some(RunningTask::PickBundleOpen(task));
    }
    pub fn open_path_dialog(&mut self, runner: &mut AsyncTaskRunner) {
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog;
            let folder = AsyncFileDialog::new().pick_folder().await;
            folder.map(|f| f.path().to_string_lossy().into_owned().into())
        });
        runner.task = Some(RunningTask::PickFileOpen(task));
    }
    pub fn open_save_dialog(&mut self, runner: &mut AsyncTaskRunner) {
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog;
            let file = AsyncFileDialog::new()
                .add_filter("Rizline Chart", &["json"])
                .add_filter("Rizlium Chart", &["rzlm"])
                .add_filter("Rizlium Data", &["rzd"])
                .save_file()
                .await;
            file.map(|f| f.path().to_path_buf())
        });
        runner.task = Some(RunningTask::PickFileSave(task));
    }
}
