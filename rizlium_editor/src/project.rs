use crate::time_and_audio::GameAudioSource;
use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use bevy_kira_audio::{prelude::StaticSoundData, AudioSource};
use futures_lite::io::AsyncWriteExt;
use helium_framework::{
    prelude::{Actions, ActionsExt, ToastsStorage},
    utils::identifier::Identifier,
};
use indexmap::IndexSet;
use rizlium_chart::prelude::*;
use rizlium_render::ChartProvider;
use rust_i18n::t;
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
            .init_resource::<PendingSave>()
            .add_event::<LoadChartEvent>()
            .add_event::<ChartLoadingEvent>()
            .add_event::<SaveChartEvent>()
            .add_systems(
                PostUpdate,
                (
                    handle_load_chart_events,
                    handle_dialog_pending,
                    handle_chart_loading,
                    handle_save_chart_events,
                    handle_save_result,
                    process_loading_results,
                ),
            );
        app.reflect_system("project.load_path", "Load a path", load_path_action);
        app.reflect_system("project.load_bundle", "Load a bundle", load_bundle_action);
    }
}

fn load_path_action(In(path): In<String>, mut events: EventWriter<LoadChartEvent>) {
    events.write(LoadChartEvent::Path(path));
}
fn load_bundle_action(In(path): In<String>, mut events: EventWriter<LoadChartEvent>) {
    events.write(LoadChartEvent::Bundle(path));
}

pub enum DialogPending {
    Bundle(Task<Option<String>>),
    Path(Task<Option<String>>),
}
pub struct ChartLoading {
    pub task: Task<Result<LoadedChart, ChartLoadingError>>,
}
#[derive(Resource, Default)]
pub enum ProjectState {
    #[default]
    Idle,
    DialogPending(DialogPending),
    ChartLoading(ChartLoading),
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
    Bundle(PathBuf, Chart),
}
#[derive(Event, Debug)]
pub enum LoadChartEvent {
    Bundle(String),
    Path(String),
}
#[derive(Event)]
pub enum ChartLoadingEvent {
    Success(String),
    Error(ChartLoadingError),
}
#[derive(Event)]
pub struct SaveChartEvent;
#[derive(Debug, snafu::Snafu)]
pub enum ChartLoadingError {
    #[snafu(display("No info file present"))]
    NoInfo { path: String },
    #[snafu(display("Failed to unzip file"), context(false))]
    UnzipFileFailed { source: zip::result::ZipError },
    #[snafu(display("Failed to read file"), context(false))]
    ReadingFileFailed { source: std::io::Error },
    #[snafu(display("Chart format is invalid"), context(false))]
    ChartFormatInvalid { source: serde_json::Error },
    #[snafu(display("Chart info format is invalid"), context(false))]
    InfoFormatInvalid { source: serde_yaml::Error },
    #[snafu(display("Failed to convert chart"), context(false))]
    ChartConvertingFailed {
        source: rizlium_chart::parse::ConvertError,
    },
    #[snafu(display("Failed to convert music"), context(false))]
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
#[derive(Resource, Serialize, Deserialize, Debug, Deref, DerefMut)]
pub struct RecentFiles(#[deref] IndexSet<String>, usize);

impl Default for RecentFiles {
    fn default() -> Self {
        Self(IndexSet::new(), 4)
    }
}

impl RecentFiles {
    pub fn push(&mut self, name: String) {
        if let (idx, false) = self.0.insert_full(name.clone()) {
            let value = self.0.shift_remove_index(idx).unwrap();
            self.0.insert(value);
        }
        if self.0.len() > self.1 {
            self.0.shift_remove_index(0);
        }
    }
}
impl ChartProvider for ProjectState {
    fn chart(&self) -> &Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::DialogPending(_) => panic!("chart dialog is pending"),
            Self::ChartLoading(ChartLoading { .. }) => panic!("chart is loading"),
            Self::Loaded(project) => match project {
                LoadedProject::Folder(_, chart, _) => chart,
                LoadedProject::Bundle(_, chart) => chart,
            },
        }
    }
    fn chart_mut(&mut self) -> &mut Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::DialogPending(_) => panic!("chart dialog is pending"),
            Self::ChartLoading(ChartLoading { .. }) => panic!("chart is loading"),
            Self::Loaded(project) => match project {
                LoadedProject::Folder(_, chart, _) => chart,
                LoadedProject::Bundle(_, chart) => chart,
            },
        }
    }
    fn has_chart_system() -> impl Condition<()> {
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
                LoadedProject::Bundle(_, chart) => chart,
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
                LoadedProject::Bundle(_, chart) => chart,
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
    #[must_use]
    pub fn is_dialog_pending(&self) -> bool {
        matches!(self, Self::DialogPending(_))
    }
    #[must_use]
    pub fn is_chart_loading(&self) -> bool {
        matches!(self, Self::ChartLoading { .. })
    }
}
fn handle_load_chart_events(
    mut events: EventReader<LoadChartEvent>,
    mut state: ResMut<ProjectState>,
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
        *state = ProjectState::ChartLoading(ChartLoading { task });
    }
    events.clear();
}
fn handle_dialog_pending(mut state: ResMut<ProjectState>, mut events: EventWriter<LoadChartEvent>) {
    if !state.is_dialog_pending() {
        return;
    }
    let poll_result = match *state {
        ProjectState::DialogPending(DialogPending::Bundle(ref mut task)) => {
            let result = futures_lite::future::block_on(futures_lite::future::poll_once(task));
            result.map(|path| path.map(LoadChartEvent::Bundle))
        }
        ProjectState::DialogPending(DialogPending::Path(ref mut task)) => {
            let result = futures_lite::future::block_on(futures_lite::future::poll_once(task));
            result.map(|path| path.map(LoadChartEvent::Path))
        }
        _ => return,
    };
    let Some(selected_path) = poll_result else {
        return;
    };
    if let Some(e) = selected_path {
        events.write(e);
    } else {
        *state = ProjectState::Idle;
    }
}
fn handle_chart_loading(
    mut state: ResMut<ProjectState>,
    mut events: EventWriter<ChartLoadingEvent>,
    asset_server: Res<AssetServer>,
    mut command: Commands,
) {
    if !state.is_chart_loading() {
        return;
    }
    let ProjectState::ChartLoading(ChartLoading { ref mut task }) = *state else {
        return;
    };
    if let Some(result) = futures_lite::future::block_on(futures_lite::future::poll_once(task)) {
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
                        ));
                    }
                }
                let handle = asset_server.add(loaded.audio_source);
                command.insert_resource(GameAudioSource(handle));
                events.write(ChartLoadingEvent::Success(loaded.path));
            }
            Err(err) => {
                *state = ProjectState::Idle;
                events.write(ChartLoadingEvent::Error(err));
            }
        }
    }
}
fn handle_save_chart_events(
    mut events: EventReader<SaveChartEvent>,
    state: Res<ProjectState>,
    mut pending_save: ResMut<PendingSave>,
) {
    for _ in events.read() {
        if let ProjectState::Loaded(project) = &*state {
            let chart = match project {
                LoadedProject::Folder(_, chart, _) => chart.clone(),
                LoadedProject::Bundle(_, chart) => chart.clone(),
            };
            let path = match project {
                LoadedProject::Folder(p, _, info) => p.join(&info.chart_path),
                LoadedProject::Bundle(p, _) => p.with_extension("rzl"),
            };
            let task =
                IoTaskPool::get().spawn(async move { save_chart_to_file(&chart, &path).await });
            pending_save.task = Some(task);
        }
    }
}

fn handle_save_result(mut pending_save: ResMut<PendingSave>, mut toasts: ResMut<ToastsStorage>) {
    if let Some(mut task) = pending_save.task.take() {
        if let Some(result) =
            futures_lite::future::block_on(futures_lite::future::poll_once(&mut task))
        {
            match result {
                Ok(_) => {
                    toasts.info(t!("project.save.success"));
                }
                Err(e) => {
                    error!("Failed to save chart: {e}");
                    toasts.error(t!("project.save.fail", err = e));
                }
            }
        } else {
            pending_save.task = Some(task);
        }
    }
}
fn process_loading_results(
    mut events: EventReader<ChartLoadingEvent>,
    mut recent: ResMut<RecentFiles>,
    mut toast: ResMut<ToastsStorage>,
    mut actions: Actions,
) {
    let mut empty_folder = None;
    for event in events.read() {
        match event {
            ChartLoadingEvent::Success(path) => {
                recent.push(path.clone());
            }
            ChartLoadingEvent::Error(e) => {
                if let ChartLoadingError::NoInfo { path } = e {
                    empty_folder = Some(path.clone());
                    break;
                }
                error!("Failed to load chart: {e}");
                toast.error(t!("path.load.fail", err = e));
            }
        }
    }
    let Some(path) = empty_folder else {
        return;
    };
    actions.queue_action(&"docking.open_tab".into(), In(Identifier::from("guide")));
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
        })?;
    let info: ChartInfo = serde_yaml::from_reader(Cursor::new(info_file))?;
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
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let serialized = serde_json::to_vec_pretty(chart)?;
    let mut file = async_fs::File::create(path).await?;
    file.write_all(&serialized).await?;
    file.close().await?;
    Ok(())
}

impl ProjectState {
    pub fn open_bundle_dialog(&mut self) {
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog;
            let file = AsyncFileDialog::new()
                .add_filter("Chart Bundle", &["zip"])
                .pick_file()
                .await;
            file.map(|f| f.path().to_string_lossy().into_owned())
        });
        *self = ProjectState::DialogPending(DialogPending::Bundle(task));
    }
    pub fn open_path_dialog(&mut self) {
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog;
            let folder = AsyncFileDialog::new().pick_folder().await;
            folder.map(|f| f.path().to_string_lossy().into_owned())
        });
        *self = ProjectState::DialogPending(DialogPending::Path(task));
    }
    #[deprecated(note = "Use open_bundle_dialog instead")]
    pub fn open_dialog(&mut self) {
        self.open_bundle_dialog();
    }
}
#[derive(Resource, Default)]
struct PendingSave {
    task: Option<Task<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
}
