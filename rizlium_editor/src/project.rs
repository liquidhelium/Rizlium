use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use bevy_kira_audio::{prelude::StaticSoundData, AudioSource};
use futures_lite::io::AsyncWriteExt;
use indexmap::IndexSet;
use rizlium_chart::prelude::*;
use rizlium_render::ChartProvider;
use serde::{Deserialize, Serialize};
use snafu::ResultExt;
use std::{
    io::{Cursor, Read},
    path::{Path, PathBuf},
};
use zip::ZipArchive;

use crate::time_and_audio::GameAudioSource;

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectState>()
            .init_resource::<RecentFiles>()
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
                    report_loading_results,
                ),
            );
    }
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
    pub chart: Chart,
    pub audio_source: AudioSource,
    pub path: String,
}

#[derive(Debug, Clone)]
pub enum LoadedProject {
    Folder(PathBuf, Chart),
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

#[derive(Deserialize)]
pub struct ChartInfo {
    pub name: String,
    pub format: ChartFormat,
    pub chart_path: String,
    pub music_path: String,
}

#[derive(Deserialize)]
pub enum ChartFormat {
    Rizline,
    Rizlium,
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
                LoadedProject::Folder(_, chart) => chart,
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
                LoadedProject::Folder(_, chart) => chart,
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
            match project {
                LoadedProject::Folder(_, chart) => {
                    chart.lines.iter().map(|line| line.points.len() - 1).sum()
                }
                LoadedProject::Bundle(_, chart) => {
                    chart.lines.iter().map(|line| line.points.len() - 1).sum()
                }
            }
        } else {
            0
        }
    }

    pub fn note_count(&self) -> usize {
        if let Self::Loaded(project) = self {
            match project {
                LoadedProject::Folder(_, chart) => {
                    chart.lines.iter().map(|line| line.notes.len()).sum()
                }
                LoadedProject::Bundle(_, chart) => {
                    chart.lines.iter().map(|line| line.notes.len()).sum()
                }
            }
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

    /// Returns `true` if the project state is [`DialogPending`].
    ///
    /// [`DialogPending`]: ProjectState::DialogPending
    #[must_use]
    pub fn is_dialog_pending(&self) -> bool {
        matches!(self, Self::DialogPending(_))
    }

    /// Returns `true` if the project state is [`ChartLoading`].
    ///
    /// [`ChartLoading`]: ProjectState::ChartLoading
    #[must_use]
    pub fn is_chart_loading(&self) -> bool {
        matches!(self, Self::ChartLoading { .. })
    }
}

// 系统函数实现
fn handle_load_chart_events(
    mut events: EventReader<LoadChartEvent>,
    mut state: ResMut<ProjectState>,
) {
    if events.is_empty() {
        return;
    }

    // 只处理最后一个事件
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
            let result = futures_lite::future::block_on(futures_lite::future::poll_once(task))
;
            result.map(|path| path.map(LoadChartEvent::Bundle))
        },
        ProjectState::DialogPending(DialogPending::Path(ref mut task)) => {
            let result = futures_lite::future::block_on(futures_lite::future::poll_once(task))
;
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
                *state = ProjectState::Loaded(LoadedProject::Bundle(
                    PathBuf::from(&loaded.path),
                    loaded.chart,
                ));
                let handle = asset_server.add(loaded.audio_source);
                command.insert_resource(GameAudioSource(handle));
                events.write(ChartLoadingEvent::Success(path));
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
    mut commands: Commands,
) {
    for _ in events.read() {
        if let ProjectState::Loaded(project) = &*state {
            let chart = match project {
                LoadedProject::Folder(_, chart) => chart.clone(),
                LoadedProject::Bundle(_, chart) => chart.clone(),
            };
            let path = match project {
                LoadedProject::Folder(p, _) => p.join("chart.rzl"),
                LoadedProject::Bundle(p, _) => p.with_extension("rzl"),
            };

            let task =
                IoTaskPool::get().spawn(async move { save_chart_to_file(&chart, &path).await });

            commands.insert_resource(PendingSave { task: Some(task) });
        }
    }
}

fn report_loading_results(
    mut events: EventReader<ChartLoadingEvent>,
    mut recent: ResMut<RecentFiles>,
) {
    for event in events.read() {
        match event {
            ChartLoadingEvent::Success(path) => {
                recent.push(path.clone());
            }
            ChartLoadingEvent::Error(_) => {}
        }
    }
}

// 工具函数
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
        chart,
        audio_source,
        path: path.to_string(),
    })
}

async fn load_chart_from_path(path: &str) -> Result<LoadedChart, ChartLoadingError> {
    let path_buf = PathBuf::from(path);

    // 从文件夹加载图表
    let info_file = async_fs::read(path_buf.join("info.yml")).await?;
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
    };

    let audio_data = async_fs::read(path_buf.join(&info.music_path)).await?;
    let audio_source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(audio_data))
            .map_err(|e| ChartLoadingError::MusicConvertingFailed { source: e })?,
    };

    Ok(LoadedChart {
        chart,
        audio_source,
        path: path.to_string(),
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

// 公开API
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
            let folder = AsyncFileDialog::new()
                .pick_folder()
                .await;
            folder.map(|f| f.path().to_string_lossy().into_owned())
        });
        *self = ProjectState::DialogPending(DialogPending::Path(task));
    }

    #[deprecated(note = "Use open_bundle_dialog instead")]
    pub fn open_dialog(&mut self) {
        self.open_bundle_dialog();
    }
}

#[derive(Resource)]
struct PendingSave {
    task: Option<Task<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
}

impl Default for PendingSave {
    fn default() -> Self {
        Self { task: None }
    }
}
