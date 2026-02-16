use bevy::{prelude::*, tasks::IoTaskPool};
use egui::{Align, Color32, Context, Layout, RichText, Ui};
use futures_lite::StreamExt as _;
use helium_framework::prelude::TabRegistrationExt;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::project::{AsyncTaskRunner, ChartInfo, LoadedProject, ProjectState};

pub struct ExplorerPlugin;

impl Plugin for ExplorerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExplorerState>()
            .register_tab("explorer", "Explorer", explorer_tab, || true)
            .add_systems(Update, (update_explorer_state, handle_explorer_loading));
    }
}

#[derive(Resource, Default)]
pub struct ExplorerState {
    pub current_path: Option<PathBuf>,
    pub files: Vec<FileEntry>,
    pub is_loading: bool,
    pub last_refresh: Option<SystemTime>,
    pub creating_new: Option<NewItemState>,
    pub renaming: Option<RenameState>,
}

#[derive(Debug, Clone)]
pub struct NewItemState {
    pub name: String,
    pub is_dir: bool,
    pub temp_id: String,
}

#[derive(Debug, Clone)]
pub struct RenameState {
    pub path: PathBuf,
    pub old_name: String,
    pub new_name: String,
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub is_hidden: bool,
}

fn explorer_tab(
    InMut(mut ui): InMut<Ui>,
    mut state: ResMut<ExplorerState>,
    mut project: ResMut<ProjectState>,
    mut runner: ResMut<AsyncTaskRunner>,
    mut commands: Commands,
) {
    let ui = &mut ui;

    let current_path = state.current_path.clone();
    handle_keyboard_events(ui.ctx(), &mut state, current_path.as_ref(), &mut commands);

    // 顶部工具栏
    ui.horizontal(|ui| {
        let folder_name = state
            .current_path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("EXPLORER");
        ui.label(RichText::new(folder_name).strong());
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui.button("🔄").on_hover_text("Refresh").clicked() {
                if let Some(path) = state.current_path.clone() {
                    state.is_loading = true;
                    let task =
                        IoTaskPool::get().spawn(async move { read_folder_contents(&path).await });
                    commands.insert_resource(ExplorerLoading { task });
                }
            }
            if ui.button("📁+").on_hover_text("New Folder").clicked() {
                start_creating_new_item(&mut state, true);
            }
            if ui.button("📄+").on_hover_text("New File").clicked() {
                start_creating_new_item(&mut state, false);
            }
        });
    });

    ui.separator();

    // 主内容区域
    let current_path = state.current_path.clone();
    let files = state.files.clone();

    if let Some(path) = current_path {
        folder_view(ui, &path, &files, &mut state, &mut commands);
    } else {
        welcome_view(ui, project.into(), runner.into());
    }
}

fn welcome_view(ui: &mut Ui, mut project: Mut<ProjectState>, mut runner: Mut<AsyncTaskRunner>) {
    ui.vertical_centered(|ui| {
        ui.add_space(40.0);

        // VSCode风格的欢迎界面
        ui.label(RichText::new("🗂️").size(48.0));
        ui.add_space(10.0);
        ui.label(RichText::new("No folder opened").strong().size(16.0));
        ui.label(RichText::new("You have not yet opened a folder.").color(Color32::GRAY));

        ui.add_space(30.0);

        ui.vertical_centered(|ui| {
            ui.set_max_width(200.0);

            if ui
                .button(RichText::new("📁 Open Folder").strong().size(14.0))
                .clicked()
            {
                // 打开文件夹对话框 - 通过系统事件触发
                project.open_path_dialog(&mut runner);
            }

            ui.add_space(8.0);

            if ui
                .button(RichText::new("📦 Open Chart Bundle").size(14.0))
                .clicked()
            {
                // 打开图表包对话框 - 通过系统事件触发
                project.open_bundle_dialog(&mut runner);
            }

            ui.add_space(20.0);

            ui.label(
                RichText::new("Start by opening a folder containing your chart files")
                    .small()
                    .color(Color32::GRAY),
            );
        });
    });
}

fn folder_view(
    ui: &mut Ui,
    path: &Path,
    files: &[FileEntry],
    state: &mut ExplorerState,
    commands: &mut Commands,
) {
    // 克隆需要的状态，避免借用冲突
    let creating_new = state.creating_new.clone();
    let renaming = state.renaming.clone();

    // 文件列表
    egui::ScrollArea::vertical().show(ui, |ui| {
        // 显示正在创建的新项目
        if let Some(new_item) = creating_new {
            let icon = if new_item.is_dir { "📁" } else { "📄" };

            ui.horizontal(|ui| {
                ui.label(icon);
                let mut name = new_item.name.clone();
                let response = inline_edit(ui, &mut name, true, None);
                if response.lost_focus() {
                    let trimmed_name = name.trim();
                    if !trimmed_name.is_empty() && validate_filename(trimmed_name, files) {
                        create_new_item(path, trimmed_name, new_item.is_dir, state, commands);
                    }
                    state.creating_new = None;
                } else {
                    // 更新状态
                    state.creating_new.as_mut().unwrap().name = name;
                }
            });
        }

        // 显示现有文件
        if files.is_empty() && state.creating_new.is_none() {
            ui.label(RichText::new("No files found").color(Color32::GRAY));
        } else {
            for file in files {
                if file.is_hidden {
                    continue;
                }

                let is_renaming = renaming.as_ref().is_some_and(|r| r.path == file.path);

                if is_renaming {
                    if let Some(rename_state) = renaming.clone() {
                        ui.horizontal(|ui| {
                            let icon = if file.is_dir { "📁" } else { "📄" };
                            ui.label(icon);
                            let mut new_name = rename_state.new_name.clone();
                            let response =
                                inline_edit(ui, &mut new_name, true, Some(&rename_state.old_name));
                            if response.lost_focus() {
                                let trimmed_name = new_name.trim();
                                if !trimmed_name.is_empty()
                                    && trimmed_name != rename_state.old_name
                                    && validate_filename(trimmed_name, files)
                                {
                                    rename_item(&rename_state.path, trimmed_name, state, commands);
                                }
                                state.renaming = None;
                            } else {
                                // 更新状态
                                if let Some(rename) = state.renaming.as_mut() {
                                    if rename.path == file.path {
                                        rename.new_name = new_name;
                                    }
                                }
                            }
                        });
                    }
                } else {
                    ui.horizontal(|ui| {
                        let icon = if file.is_dir { "📁" } else { "📄" };
                        let text = RichText::new(format!(" {} {}", icon, file.name));

                        let response = ui
                            .add(egui::Label::new(text).sense(egui::Sense::click()))
                            .on_hover_text(file.path.display().to_string());

                        // 悬停高亮效果
                        if response.hovered() {
                            response.clone().highlight();
                        }

                        // 右键菜单
                        let context_menu_response = response.context_menu(|ui| {
                            if ui.button("重命名").clicked() {
                                state.renaming = Some(RenameState {
                                    path: file.path.clone(),
                                    old_name: file.name.clone(),
                                    new_name: file.name.clone(),
                                });
                                ui.close();
                            }

                            if ui.button("删除").clicked() {
                                delete_item(&file.path, state, commands);
                                ui.close();
                            }
                        });

                        // 双击重命名
                        if response.double_clicked() {
                            state.renaming = Some(RenameState {
                                path: file.path.clone(),
                                old_name: file.name.clone(),
                                new_name: file.name.clone(),
                            });
                        }
                    });
                }
            }
        }
    });
}

// 内联编辑组件 - 类似VSCode的文件名编辑
fn inline_edit(
    ui: &mut Ui,
    text: &mut String,
    selected: bool,
    original_name: Option<&str>,
) -> egui::Response {
    let id = ui.auto_id_with("inline_edit");
    let state = egui::TextEdit::singleline(text)
        .id(id)
        .desired_width(150.0)
        .show(ui);

    if selected {
        state.response.request_focus();
        // 选择全部文本
        if let Some(mut edit_state) = egui::TextEdit::load_state(ui.ctx(), id) {
            if let Some(char_range) = edit_state.cursor.char_range() {
                if char_range.secondary.index == 0 {
                    let len = text.len();
                    if let Some(original) = original_name {
                        // 如果是重命名，选择文件名（不含扩展名）
                        if let Some(dot_idx) = original.rfind('.') {
                            edit_state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::two(
                                    egui::text::CCursor::new(0),
                                    egui::text::CCursor::new(dot_idx),
                                )));
                        } else {
                            edit_state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::two(
                                    egui::text::CCursor::new(0),
                                    egui::text::CCursor::new(len),
                                )));
                        }
                    } else {
                        // 新建时选择全部文本
                        edit_state
                            .cursor
                            .set_char_range(Some(egui::text::CCursorRange::two(
                                egui::text::CCursor::new(0),
                                egui::text::CCursor::new(len),
                            )));
                    }
                    edit_state.store(ui.ctx(), id);
                }
            }
        }
    }

    state.response
}

// 验证文件名
fn validate_filename(name: &str, existing_files: &[FileEntry]) -> bool {
    if name.is_empty() || name.contains('/') || name.contains('\\') || name.contains(':') {
        return false;
    }

    // 检查是否已存在同名文件
    !existing_files.iter().any(|f| f.name == name)
}

// 创建新文件或文件夹
fn create_new_item(
    path: &Path,
    name: &str,
    is_dir: bool,
    state: &mut ExplorerState,
    commands: &mut Commands,
) {
    let new_path = path.join(name);

    if is_dir {
        if let Err(e) = std::fs::create_dir_all(&new_path) {
            warn!("Failed to create directory {}: {}", new_path.display(), e);
        }
    } else if let Err(e) = std::fs::write(&new_path, b"") {
        warn!("Failed to create file {}: {}", new_path.display(), e);
    }

    // 刷新文件列表
    state.is_loading = true;
    let path_clone = path.to_path_buf();
    let task = IoTaskPool::get().spawn(async move { read_folder_contents(&path_clone).await });
    commands.insert_resource(ExplorerLoading { task });
}

// 重命名文件或文件夹
fn rename_item(
    old_path: &Path,
    new_name: &str,
    state: &mut ExplorerState,
    commands: &mut Commands,
) {
    if let Some(parent) = old_path.parent() {
        let new_path = parent.join(new_name);

        if let Err(e) = std::fs::rename(old_path, &new_path) {
            warn!(
                "Failed to rename {} to {}: {}",
                old_path.display(),
                new_path.display(),
                e
            );
            return;
        }

        // 刷新文件列表
        state.is_loading = true;
        let path_clone = parent.to_path_buf();
        let task = IoTaskPool::get().spawn(async move { read_folder_contents(&path_clone).await });
        commands.insert_resource(ExplorerLoading { task });
    }
}

// 删除文件或文件夹
fn delete_item(path: &Path, state: &mut ExplorerState, commands: &mut Commands) {
    if let Some(parent) = path.parent() {
        if path.is_dir() {
            if let Err(e) = std::fs::remove_dir_all(path) {
                warn!("Failed to delete directory {}: {}", path.display(), e);
                return;
            }
        } else if let Err(e) = std::fs::remove_file(path) {
            warn!("Failed to delete file {}: {}", path.display(), e);
            return;
        }

        // 刷新文件列表
        state.is_loading = true;
        let path_clone = parent.to_path_buf();
        let task = IoTaskPool::get().spawn(async move { read_folder_contents(&path_clone).await });
        commands.insert_resource(ExplorerLoading { task });
    }
}

// 辅助函数：读取文件夹内容
pub async fn read_folder_contents(path: &Path) -> std::io::Result<Vec<FileEntry>> {
    use async_fs::read_dir;

    let mut files = Vec::new();
    let mut entries = read_dir(path).await?;

    while let Some(entry) = entries.next().await {
        if let Ok(entry) = entry {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type().await.map(|t| t.is_dir()).unwrap_or(false);
            let is_hidden = name.starts_with('.');

            files.push(FileEntry {
                name,
                path,
                is_dir,
                is_hidden,
            });
        }
    }

    // 排序：文件夹在前，然后按名称排序
    files.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.cmp(&b.name),
    });

    Ok(files)
}

// 更新explorer状态的系统
pub fn update_explorer_state(
    mut explorer: ResMut<ExplorerState>,
    project: Res<ProjectState>,
    mut commands: Commands,
) {
    let new_path = match &*project {
        ProjectState::Loaded(LoadedProject::Folder(path, _, _)) => Some(path.clone()),
        _ => None,
    };

    if explorer.current_path != new_path {
        explorer.current_path = new_path.clone();
        if let Some(path) = new_path {
            explorer.is_loading = true;
            explorer.files.clear();

            let task = IoTaskPool::get().spawn(async move { read_folder_contents(&path).await });
            commands.insert_resource(ExplorerLoading { task });
        }
    }
}

#[derive(Resource)]
pub struct ExplorerLoading {
    pub task: bevy::tasks::Task<std::io::Result<Vec<FileEntry>>>,
}

// 处理键盘事件
fn handle_keyboard_events(
    ctx: &Context,
    state: &mut ExplorerState,
    current_path: Option<&PathBuf>,
    commands: &mut Commands,
) {
    let mut should_confirm_create = false;
    let mut should_confirm_rename = false;

    ctx.input(|input| {
        if input.key_pressed(egui::Key::Enter) {
            if state.creating_new.is_some() {
                should_confirm_create = true;
            } else if state.renaming.is_some() {
                should_confirm_rename = true;
            }
        }

        if input.key_pressed(egui::Key::Escape) {
            state.creating_new = None;
            state.renaming = None;
        }
    });

    // 处理确认事件（在input闭包外处理，避免借用冲突）
    if should_confirm_create {
        if let Some(new_item) = state.creating_new.take() {
            let name = new_item.name.trim();
            if !name.is_empty() {
                if let Some(path) = current_path {
                    create_new_item(path, name, new_item.is_dir, state, commands);
                }
            }
        }
    }

    if should_confirm_rename {
        if let Some(rename_state) = state.renaming.take() {
            let new_name = rename_state.new_name.trim();
            if !new_name.is_empty() && new_name != rename_state.old_name {
                rename_item(&rename_state.path, new_name, state, commands);
            }
        }
    }
}

// 开始创建新项目
fn start_creating_new_item(state: &mut ExplorerState, is_dir: bool) {
    let temp_id = uuid::Uuid::new_v4().to_string();
    let default_name = if is_dir { "New Folder" } else { "New File" };

    // 确保名称唯一
    let mut counter = 1;
    let mut name = default_name.to_string();
    while state.files.iter().any(|f| f.name == name) {
        name = format!("{default_name}{counter}");
        counter += 1;
    }

    state.creating_new = Some(NewItemState {
        name,
        is_dir,
        temp_id,
    });
}

// 处理explorer加载完成的系统
pub fn handle_explorer_loading(
    mut commands: Commands,
    mut explorer: ResMut<ExplorerState>,
    loading: Option<ResMut<ExplorerLoading>>,
) {
    let Some(mut loading) = loading else { return };

    if let Some(result) =
        futures_lite::future::block_on(futures_lite::future::poll_once(&mut loading.task))
    {
        match result {
            Ok(files) => {
                explorer.files = files;
                explorer.last_refresh = Some(SystemTime::now());
            }
            Err(_) => {
                explorer.files.clear();
            }
        }
        explorer.is_loading = false;
        commands.remove_resource::<ExplorerLoading>();
    }
}
