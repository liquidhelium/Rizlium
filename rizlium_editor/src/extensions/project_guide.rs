use std::{future::Future, io::Write, pin::pin};

use bevy::{
    ecs::system::{InMut, Local},
    prelude::*,
    tasks::Task,
};
use egui::{Ui, Widget};
use futures_lite::future::{block_on, poll_once};
use helium_framework::{prelude::*, utils::identifier::Identifier};
use rfd::{AsyncFileDialog, FileHandle};
use rizlium_chart::chart::Chart;

pub struct ProjectGuideExtension;

impl Plugin for ProjectGuideExtension {
    fn build(&self, app: &mut App) {
        app.register_tab("guide", "Project Guide", project_guide, || true);
    }
}

#[derive(Default)]
struct Tasks {
    select_folder: Option<Task<Option<FileHandle>>>,
    select_song: Option<Task<Option<FileHandle>>>,
}
fn project_guide(
    InMut(ui): InMut<Ui>,
    mut selected_path: Local<Option<String>>,
    mut selected_song: Local<Option<String>>,
    mut tasks: Local<Tasks>,
    mut actions: Actions,
    mut toasts: ResMut<ToastsStorage>,
) {
    if selected_path.is_none() {
        select_path(ui, &mut selected_path, &mut tasks, &mut toasts);
    } else {
        ui.heading("Create a new project");
        ui.label("2. Create project structure");
        let mut song_select = FileSelectWidget::new("Select a song file");
        song_select.ui(ui, &mut tasks.select_song, &mut selected_song);

        if ui
            .add_enabled(
                selected_song.is_some(),
                egui::Button::new("Create Project"),
            )
            .clicked()
        {
            if let Some(path) = selected_path.as_deref() {
                if let Some(song) = selected_song.as_deref() {
                    match create_project_structure(path, song) {
                        Ok(_) => {
                            toasts.success("Project created successfully.");
                        }
                        Err(e) => {
                            toasts.error(format!("Failed to create project: {}", e));
                        }
                    }
                    actions.queue_action(&"project.load_path".into(), In(path.to_string()));
                }
            }
            else {
                toasts.error("No path selected.");
            }
            actions.queue_action(&"docking.close_tab".into(), In(Identifier::from("guide")));
        }
        if ui.button("Back").clicked() {
            *selected_path = None;
        }
    }
}

fn create_file(path: &str, content: &str) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
    file.write(content.as_bytes())?;
    Ok(())
}

fn create_project_structure(path: &str, song: &str) -> std::io::Result<()> {
    // Copy song file
    let song_filename = std::path::Path::new(song)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid song file name",
        ))?;
    std::fs::copy(song, &format!("{}/{}", path, song_filename))?;
    // Create info.yml
    create_file(
        &format!("{}/info.yml", path),
        &format!(
            r#"name: New Project
format: Rizlium
chart_path: chart.rzlm
music_path: {}"#,
            song_filename
        ),
    )?;
    // Create default chart file
    create_file(&format!("{}/chart.rzlm", path), serde_json::to_string_pretty(&Chart::empty()).unwrap().as_str())?;
    Ok(())
}

fn select_path(
    ui: &mut Ui,
    selected_path: &mut Local<'_, Option<String>>,
    selected_path_task: &mut Local<'_, Tasks>,
    toasts: &mut ResMut<'_, ToastsStorage>,
) {
    ui.heading("Create a new project");
    ui.label("1. Select an empty folder");
    if ui.button("Select Folder").clicked() && selected_path_task.select_folder.is_none() {
        selected_path_task.select_folder = Some(bevy::tasks::IoTaskPool::get().spawn(async {
            AsyncFileDialog::new()
                .set_title("Select an empty folder")
                .pick_folder()
                .await
        }));
    }
    if let Some(task) = selected_path_task.select_folder.as_mut() {
        if let Some(result) = futures_lite::future::block_on(poll_once(task)) {
            match result {
                Some(handle) => {
                    let path = handle.path().to_string_lossy().to_string();
                    if std::fs::read_dir(&path)
                        .map(|mut i| i.next().is_none())
                        .unwrap_or(false)
                    {
                        **selected_path = Some(path);
                    } else {
                        toasts.error("The selected folder is not empty.");
                    }
                }
                _ => (),
            }
            selected_path_task.select_folder = None;
        }
    }
}

struct FileSelectWidget {
    label: &'static str,
}

impl FileSelectWidget {
    fn ui(
        &mut self,
        ui: &mut Ui,
        task: &mut Option<Task<Option<FileHandle>>>,
        selected_file: &mut Option<String>,
    ) {
        let mut changed = false;
        ui.horizontal(|ui| {
            ui.label(self.label);
            if let Some(file) = selected_file {
                ui.label(file.as_str());
            } else {
                ui.label("No file selected");
            }
            if ui.button("Select File").clicked() && task.is_none() {
                *task = Some(bevy::tasks::IoTaskPool::get().spawn(async {
                    AsyncFileDialog::new()
                        .add_filter("Audio", &["mp3", "wav", "ogg"])
                        .set_title("Select a song file")
                        .pick_file()
                        .await
                }));
            }
            if let Some(task_inner) = task.as_mut() {
                if let Some(result) = futures_lite::future::block_on(poll_once(task_inner)) {
                    match result {
                        Some(handle) => {
                            let path = handle.path().to_string_lossy().to_string();
                            *selected_file = Some(path);
                            changed = true;
                        }
                        _ => (),
                    }
                    task.take();
                }
            }
        });
    }
}

impl FileSelectWidget {
    fn new(label: &'static str) -> Self {
        Self { label }
    }
}
