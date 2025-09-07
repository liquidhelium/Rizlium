// 引入 Bevy 引擎的核心模块。
use bevy::{
    prelude::*, // 引入 Bevy 预设，包含常用功能。
    tasks::{IoTaskPool, Task}, // 引入 IO 任务池和任务类型，用于异步执行文件操作。
};
// 引入 `bevy_kira_audio` 插件，用于处理音频。
use bevy_kira_audio::{prelude::StaticSoundData, AudioSource};
// 引入 `futures_lite` 的异步写操作 trait。
use futures_lite::io::AsyncWriteExt;
// 引入 `IndexSet`，一个保持插入顺序的哈希集合，用于管理最近文件列表。
use indexmap::IndexSet;
// 引入 `rizlium_chart` crate 的预设模块，包含谱面相关的所有定义。
use rizlium_chart::prelude::*;
// 引入 `rizlium_render` crate 的 `ChartProvider` trait，用于让渲染系统能访问谱面数据。
use rizlium_render::ChartProvider;
// 引入 `serde`，用于序列化和反序列化数据。
use serde::{Deserialize, Serialize};
// 引入标准库中的模块。
use std::{
    io::{Cursor, Read}, // `Cursor` 用于在内存中的字节数组上进行读写，`Read` 是读取操作的 trait。
    path::{Path, PathBuf}, // `Path` 和 `PathBuf` 用于处理文件系统路径。
};
// 引入 `zip` crate，用于处理 zip 压缩文件。
use zip::ZipArchive;

// 引入当前 crate 中的 `GameAudioSource` 资源，用于存储游戏音频的句柄。
use crate::time_and_audio::GameAudioSource;

/// `ProjectPlugin` 是负责项目管理功能的 Bevy 插件。
pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    /// `build` 方法用于向 Bevy 应用中添加资源、事件和系统。
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectState>() // 初始化项目状态资源。
            .init_resource::<RecentFiles>() // 初始化最近文件列表资源。
            .add_event::<LoadChartEvent>() // 注册加载谱面事件。
            .add_event::<ChartLoadingEvent>() // 注册谱面加载中事件。
            .add_event::<SaveChartEvent>() // 注册保存谱面事件。
            .add_systems(
                PostUpdate, // 在每次更新循环的 `PostUpdate` 阶段运行这些系统。
                (
                    handle_load_chart_events, // 处理加载谱面请求。
                    handle_dialog_pending, // 处理文件对话框的异步结果。
                    handle_chart_loading, // 处理正在加载的谱面的异步任务。
                    handle_save_chart_events, // 处理保存谱面的请求。
                    report_loading_results, // 报告加载结果（成功或失败）。
                ),
            );
    }
}

/// `DialogPending` 枚举表示一个正在等待用户选择文件的异步文件对话框。
pub enum DialogPending {
    // 等待选择一个谱面包（.zip 文件）。
    Bundle(Task<Option<String>>),
    // 等待选择一个包含谱面文件的文件夹。
    Path(Task<Option<String>>),
}

/// `ChartLoading` 结构体表示一个正在从磁盘异步加载谱面的任务。
pub struct ChartLoading {
    // `Task` 包含了异步加载操作的句柄，其结果是 `Result<LoadedChart, ChartLoadingError>`。
    pub task: Task<Result<LoadedChart, ChartLoadingError>>,
}

/// `ProjectState` 是一个 Bevy 资源，同时也是一个状态机，表示当前项目的状态。
#[derive(Resource, Default)]
pub enum ProjectState {
    /// `Idle` 状态表示当前没有加载任何项目。这是默认状态。
    #[default]
    Idle,
    /// `DialogPending` 状态表示正在等待用户通过文件对话框选择文件。
    DialogPending(DialogPending),
    /// `ChartLoading` 状态表示谱面正在异步加载中。
    ChartLoading(ChartLoading),
    /// `Loaded` 状态表示项目已成功加载到内存中。
    Loaded(LoadedProject),
}

/// `LoadedChart` 结构体包含了从磁盘成功加载的所有谱面相关数据。
#[derive(Clone)]
pub struct LoadedChart {
    // 谱面的来源类型（文件夹或压缩包）。
    pub source: SourceKind,
    // 解析后的谱面数据。
    pub chart: Chart,
    // 加载后的音频数据源。
    pub audio_source: AudioSource,
    // 谱面的文件路径。
    pub path: String,
}
/// `SourceKind` 枚举表示谱面的来源。
#[derive(Clone)]
pub enum SourceKind {
    Folder, // 来自一个文件夹。
    Bundle, // 来自一个 .zip 压缩包。
}

/// `LoadedProject` 枚举表示加载到内存中的项目数据。
#[derive(Debug, Clone)]
pub enum LoadedProject {
    // 从文件夹加载的项目，包含文件夹路径和谱面数据。
    Folder(PathBuf, Chart),
    // 从谱面包加载的项目，包含 .zip 文件路径和谱面数据。
    Bundle(PathBuf, Chart),
}

/// `LoadChartEvent` 是一个 Bevy 事件，用于触发谱面加载流程。
#[derive(Event, Debug)]
pub enum LoadChartEvent {
    // 请求从指定的 .zip 谱面包路径加载。
    Bundle(String),
    // 请求从指定的文件夹路径加载。
    Path(String),
}

/// `ChartLoadingEvent` 是一个 Bevy 事件，用于报告谱面加载的结果。
#[derive(Event)]
pub enum ChartLoadingEvent {
    // 表示谱面成功加载，并附带其路径。
    Success(String),
    // 表示谱面加载失败，并附带错误信息。
    Error(ChartLoadingError),
}

/// `SaveChartEvent` 是一个 Bevy 事件，用于触发保存当前谱面的操作。
#[derive(Event)]
pub struct SaveChartEvent;

/// `ChartLoadingError` 枚举定义了在加载谱面过程中可能发生的所有错误。
/// `snafu` 是一个用于创建结构化错误的库。
#[derive(Debug, snafu::Snafu)]
pub enum ChartLoadingError {
    /// 解压文件失败。
    #[snafu(display("Failed to unzip file"), context(false))]
    UnzipFileFailed { source: zip::result::ZipError },
    /// 读取文件失败。
    #[snafu(display("Failed to read file"), context(false))]
    ReadingFileFailed { source: std::io::Error },
    /// 谱面文件（如 .rzl）格式无效。
    #[snafu(display("Chart format is invalid"), context(false))]
    ChartFormatInvalid { source: serde_json::Error },
    /// `info.yml` 配置文件格式无效。
    #[snafu(display("Chart info format is invalid"), context(false))]
    InfoFormatInvalid { source: serde_yaml::Error },
    /// 从 `Rizline` 格式转换为内部 `Chart` 格式失败。
    #[snafu(display("Failed to convert chart"), context(false))]
    ChartConvertingFailed {
        source: rizlium_chart::parse::ConvertError,
    },
    /// 加载或解码音乐文件失败。
    #[snafu(display("Failed to convert music"), context(false))]
    MusicConvertingFailed {
        source: bevy_kira_audio::prelude::FromFileError,
    },
}

/// `ChartInfo` 结构体对应于 `info.yml` 文件的内容。
#[derive(Deserialize)]
pub struct ChartInfo {
    pub name: String, // 谱面名称。
    pub format: ChartFormat, // 谱面格式。
    pub chart_path: String, // 谱面文件相对于 `info.yml` 的路径。
    pub music_path: String, // 音乐文件相对于 `info.yml` 的路径。
}

/// `ChartFormat` 枚举定义了支持的谱面格式。
#[derive(Deserialize)]
pub enum ChartFormat {
    Rizline, // 旧版格式。
    Rizlium, // 当前标准格式。
}

/// `RecentFiles` 是一个 Bevy 资源，用于存储最近打开的文件列表。
/// 它包装了一个 `IndexSet` 来保持顺序并自动去重，以及一个 `usize` 来限制列表的最大长度。
#[derive(Resource, Serialize, Deserialize, Debug, Deref, DerefMut)]
pub struct RecentFiles(#[deref] IndexSet<String>, usize);

impl Default for RecentFiles {
    fn default() -> Self {
        // 默认最多存储 4 个最近文件。
        Self(IndexSet::new(), 4)
    }
}

impl RecentFiles {
    /// 将一个文件路径添加到最近文件列表中。
    pub fn push(&mut self, name: String) {
        // `insert_full` 会返回元素是否已存在。
        // 如果路径已存在，我们先移除旧的，再在末尾插入新的，以确保它被更新到“最新”的位置。
        if let (idx, false) = self.0.insert_full(name.clone()) {
            let value = self.0.shift_remove_index(idx).unwrap();
            self.0.insert(value);
        }
        // 如果列表超出了最大长度，则移除最旧的条目（位于索引 0）。
        if self.0.len() > self.1 {
            self.0.shift_remove_index(0);
        }
    }
}

/// 为 `ProjectState` 实现 `ChartProvider` trait。
/// 这使得渲染系统可以通过一个统一的接口来访问当前加载的谱面数据，
/// 而无需关心 `ProjectState` 内部的具体状态。
impl ChartProvider for ProjectState {
    /// 返回对当前谱面的不可变引用。
    /// 如果当前没有加载谱面，或者正在加载中，则会 panic。
    /// 这是因为调用此方法的系统应该在 `has_chart_system` 条件为真时才运行。
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

    /// 返回对当前谱面的可变引用。
    /// 同样，在不合适的项目状态下调用会 panic。
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

    /// 返回一个 Bevy 系统条件（`Condition`），该条件仅在谱面已成功加载时为 `true`。
    /// 这可以用于 `.run_if()` 中，以确保某些系统只在有谱面可供处理时才运行。
    fn has_chart_system() -> impl Condition<()> {
        IntoSystem::into_system(|state: Res<ProjectState>| {
            matches!(*state, ProjectState::Loaded(_))
        })
    }
}

impl ProjectState {
    /// 计算并返回当前谱面中所有线段（segment）的总数。
    pub fn segment_count(&self) -> usize {
        if let Self::Loaded(project) = self {
            let chart = match project {
                LoadedProject::Folder(_, chart) => chart,
                LoadedProject::Bundle(_, chart) => chart,
            };
            // 每条线有两个点，所以线段数是点数减一。
            chart.lines.iter().map(|line| line.points.len().saturating_sub(1)).sum()
        } else {
            0
        }
    }

    /// 计算并返回当前谱面中所有音符（note）的总数。
    pub fn note_count(&self) -> usize {
        if let Self::Loaded(project) = self {
            let chart = match project {
                LoadedProject::Folder(_, chart) => chart,
                LoadedProject::Bundle(_, chart) => chart,
            };
            chart.lines.iter().map(|line| line.notes.len()).sum()
        } else {
            0
        }
    }

    /// 检查当前是否有谱面已加载。
    pub fn has_chart(&self) -> bool {
        matches!(self, Self::Loaded(_))
    }

    /// 如果项目已加载，则返回对 `LoadedProject` 的引用。
    pub fn loaded_project(&self) -> Option<&LoadedProject> {
        match self {
            Self::Loaded(project) => Some(project),
            _ => None,
        }
    }

    /// 检查当前项目状态是否为 `DialogPending`。
    #[must_use]
    pub fn is_dialog_pending(&self) -> bool {
        matches!(self, Self::DialogPending(_))
    }

    /// 检查当前项目状态是否为 `ChartLoading`。
    #[must_use]
    pub fn is_chart_loading(&self) -> bool {
        matches!(self, Self::ChartLoading { .. })
    }
}

// --- 系统函数实现 ---

/// `handle_load_chart_events` 系统负责处理 `LoadChartEvent` 事件。
fn handle_load_chart_events(
    mut events: EventReader<LoadChartEvent>, // 读取加载事件。
    mut state: ResMut<ProjectState>, // 可变地访问项目状态。
) {
    if events.is_empty() {
        return;
    }

    // 如果在一帧内有多个加载事件，只处理最后一个，以避免不必要的工作。
    if let Some(event) = events.read().last() {
        info!("attempting to load chart {event:?}");
        // 根据事件类型，创建一个异步任务来加载谱面。
        let task = match event {
            LoadChartEvent::Bundle(path) => {
                let path = path.clone();
                // 使用 `IoTaskPool` 在后台线程池中执行文件 IO 操作，避免阻塞主线程。
                IoTaskPool::get().spawn(async move { load_chart_from_bundle(&path).await })
            }
            LoadChartEvent::Path(path) => {
                let path = path.clone();
                IoTaskPool::get().spawn(async move { load_chart_from_path(&path).await })
            }
        };
        // 将项目状态切换到 `ChartLoading`，并存储异步任务的句柄。
        *state = ProjectState::ChartLoading(ChartLoading { task });
    }
    // 清空事件队列，因为我们已经处理了它们。
    events.clear();
}

/// `handle_dialog_pending` 系统负责轮询文件对话框的异步任务。
fn handle_dialog_pending(mut state: ResMut<ProjectState>, mut events: EventWriter<LoadChartEvent>) {
    // 如果状态不是 `DialogPending`，则直接返回。
    if !state.is_dialog_pending() {
        return;
    }
    // 从状态中取出异步任务并轮询一次。
    let poll_result = match *state {
        ProjectState::DialogPending(DialogPending::Bundle(ref mut task)) => {
            // `block_on` 和 `poll_once` 用于非阻塞地检查任务是否已完成。
            let result = futures_lite::future::block_on(futures_lite::future::poll_once(task));
            // 如果任务完成，将结果（文件路径）包装成 `LoadChartEvent`。
            result.map(|path| path.map(LoadChartEvent::Bundle))
        },
        ProjectState::DialogPending(DialogPending::Path(ref mut task)) => {
            let result = futures_lite::future::block_on(futures_lite::future::poll_once(task));
            result.map(|path| path.map(LoadChartEvent::Path))
        }
        _ => return,
    };
    // 如果任务尚未完成，则 `poll_result` 为 `None`，直接返回。
    let Some(selected_path) = poll_result else {
        return;
    };

    // 如果用户选择了文件（`Some(path)`），则发送 `LoadChartEvent` 来触发加载。
    if let Some(e) = selected_path {
        events.send(e);
    } else {
        // 如果用户取消了对话框（`None`），则将状态重置为 `Idle`。
        *state = ProjectState::Idle;
    }
}

/// `handle_chart_loading` 系统负责轮询正在加载的谱面任务。
fn handle_chart_loading(
    mut state: ResMut<ProjectState>,
    mut events: EventWriter<ChartLoadingEvent>,
    asset_server: Res<AssetServer>,
    mut command: Commands,
) {
    if !state.is_chart_loading() {
        return;
    }
    // 模式匹配以获取对异步任务的可变引用。
    let ProjectState::ChartLoading(ChartLoading { ref mut task }) = *state else {
        return;
    };

    // 轮询任务是否完成。
    if let Some(result) = futures_lite::future::block_on(futures_lite::future::poll_once(task)) {
        match result {
            // 如果加载成功...
            Ok(loaded) => {
                let path = loaded.path.clone();
                // 根据来源类型，更新项目状态为 `Loaded`。
                match loaded.source {
                    SourceKind::Folder => {
                        *state = ProjectState::Loaded(LoadedProject::Folder(
                            PathBuf::from(path),
                            loaded.chart,
                        ));
                    }
                    SourceKind::Bundle => {
                        *state = ProjectState::Loaded(LoadedProject::Bundle(
                            PathBuf::from(path),
                            loaded.chart,
                        ));
                    }
                }
                // 将加载的音频数据添加到 Bevy 的 `AssetServer` 中，获取一个句柄。
                let handle = asset_server.add(loaded.audio_source);
                // 将音频句柄存储在 `GameAudioSource` 资源中，以便音频系统可以播放它。
                command.insert_resource(GameAudioSource(handle));
                // 发送成功事件。
                events.send(ChartLoadingEvent::Success(loaded.path));
            }
            // 如果加载失败...
            Err(err) => {
                // 将状态重置为 `Idle`。
                *state = ProjectState::Idle;
                // 发送失败事件。
                events.send(ChartLoadingEvent::Error(err));
            }
        }
    }
}

/// `handle_save_chart_events` 系统负责处理 `SaveChartEvent` 事件。
fn handle_save_chart_events(
    mut events: EventReader<SaveChartEvent>,
    state: Res<ProjectState>,
    mut commands: Commands,
) {
    for _ in events.read() {
        // 确保当前有已加载的项目。
        if let ProjectState::Loaded(project) = &*state {
            // 克隆谱面数据以用于异步任务。
            let chart = match project {
                LoadedProject::Folder(_, chart) => chart.clone(),
                LoadedProject::Bundle(_, chart) => chart.clone(),
            };
            // 确定保存路径。
            let path = match project {
                LoadedProject::Folder(p, _) => p.join("chart.rzl"),
                LoadedProject::Bundle(p, _) => p.with_extension("rzl"), // 注意：这会覆盖 .zip 文件，可能不是预期行为。
            };

            // 创建一个异步任务来保存文件。
            let task =
                IoTaskPool::get().spawn(async move { save_chart_to_file(&chart, &path).await });

            // 将保存任务存储在一个资源中，以便可以跟踪其状态（虽然当前代码没有这样做）。
            commands.insert_resource(PendingSave { task: Some(task) });
        }
    }
}

/// `report_loading_results` 系统用于在加载成功后更新最近文件列表。
fn report_loading_results(
    mut events: EventReader<ChartLoadingEvent>,
    mut recent: ResMut<RecentFiles>,
) {
    for event in events.read() {
        match event {
            ChartLoadingEvent::Success(path) => {
                // 如果加载成功，将路径添加到最近文件列表。
                recent.push(path.clone());
            }
            ChartLoadingEvent::Error(_) => {} // 加载失败时不执行任何操作。
        }
    }
}

// --- 工具函数 ---

/// 从 .zip 谱面包异步加载谱面。
async fn load_chart_from_bundle(path: &str) -> Result<LoadedChart, ChartLoadingError> {
    // 异步读取整个 .zip 文件到内存。
    let file = async_fs::read(path).await?;
    // 使用 `Cursor` 在内存中的字节数组上创建 `ZipArchive`。
    let mut archive = ZipArchive::new(Cursor::new(&file))?;

    // 从压缩包中按名称查找并读取 `info.yml`。
    let info_file = archive.by_name("info.yml")?;
    // 使用 `serde_yaml` 解析 info 文件。
    let info: ChartInfo = serde_yaml::from_reader(info_file)?;

    // 根据 info 文件中指定的格式来解析谱面文件。
    let chart = match info.format {
        ChartFormat::Rizline => {
            let rzl_chart: RizlineChart =
                serde_json::from_reader(archive.by_name(&info.chart_path)?)?;
            // 将旧格式转换为内部标准格式。
            rzl_chart
                .try_into()
                .map_err(|e| ChartLoadingError::ChartConvertingFailed { source: e })?
        }
        ChartFormat::Rizlium => serde_json::from_reader(archive.by_name(&info.chart_path)?)?,
    };

    // 从压缩包中读取整个音乐文件到内存中的 `Vec<u8>`。
    let mut audio_data = Vec::new();
    archive
        .by_name(&info.music_path)?
        .read_to_end(&mut audio_data)?;
    // 从内存中的音频数据创建 `AudioSource`。
    let audio_source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(audio_data))
            .map_err(|e| ChartLoadingError::MusicConvertingFailed { source: e })?,
    };

    // 返回包含所有加载数据的 `LoadedChart` 结构体。
    Ok(LoadedChart {
        source: SourceKind::Bundle,
        chart,
        audio_source,
        path: path.to_string(),
    })
}

/// 从文件夹异步加载谱面。
async fn load_chart_from_path(path: &str) -> Result<LoadedChart, ChartLoadingError> {
    let path_buf = PathBuf::from(path);

    // 异步读取并解析 `info.yml`。
    let info_file = async_fs::read(path_buf.join("info.yml")).await?;
    let info: ChartInfo = serde_yaml::from_reader(Cursor::new(info_file))?;

    // 根据格式异步读取并解析谱面文件。
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

    // 异步读取音频文件并创建 `AudioSource`。
    let audio_data = async_fs::read(path_buf.join(&info.music_path)).await?;
    let audio_source = AudioSource {
        sound: StaticSoundData::from_cursor(Cursor::new(audio_data))
            .map_err(|e| ChartLoadingError::MusicConvertingFailed { source: e })?,
    };

    // 返回结果。
    Ok(LoadedChart {
        source: SourceKind::Folder,
        chart,
        audio_source,
        path: path.to_string(),
    })
}

/// 将谱面数据异步保存到文件。
async fn save_chart_to_file(
    chart: &Chart,
    path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // 将 `Chart` 结构体序列化为格式化的 JSON 字符串。
    let serialized = serde_json::to_vec_pretty(chart)?;
    // 异步创建或覆盖文件。
    let mut file = async_fs::File::create(path).await?;
    // 将序列化的数据写入文件。
    file.write_all(&serialized).await?;
    // 关闭文件句柄。
    file.close().await?;
    Ok(())
}

// --- 公开API ---

impl ProjectState {
    /// 打开一个异步文件对话框，让用户选择一个谱面包（.zip 文件）。
    pub fn open_bundle_dialog(&mut self) {
        // 创建一个异步任务来显示对话框。
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog; // `rfd` 是一个跨平台的文件对话框库。
            let file = AsyncFileDialog::new()
                .add_filter("Chart Bundle", &["zip"]) // 设置文件过滤器。
                .pick_file() // 显示“选择文件”对话框。
                .await; // 等待用户操作。
            // 如果用户选择了文件，则返回其路径字符串。
            file.map(|f| f.path().to_string_lossy().into_owned())
        });
        // 将项目状态切换到 `DialogPending`。
        *self = ProjectState::DialogPending(DialogPending::Bundle(task));
    }

    /// 打开一个异步文件对话框，让用户选择一个文件夹。
    pub fn open_path_dialog(&mut self) {
        let task = IoTaskPool::get().spawn(async {
            use rfd::AsyncFileDialog;
            let folder = AsyncFileDialog::new()
                .pick_folder() // 显示“选择文件夹”对话框。
                .await;
            folder.map(|f| f.path().to_string_lossy().into_owned())
        });
        *self = ProjectState::DialogPending(DialogPending::Path(task));
    }

    /// 一个已废弃的函数，为了向后兼容而保留。
    #[deprecated(note = "Use open_bundle_dialog instead")]
    pub fn open_dialog(&mut self) {
        self.open_bundle_dialog();
    }
}

/// `PendingSave` 是一个资源，用于存储正在进行的异步保存任务。
#[derive(Resource, Default)]
struct PendingSave {
    // `Option` 用于表示可能没有正在进行的保存任务。
    task: Option<Task<Result<(), Box<dyn std::error::Error + Send + Sync>>>>,
}
