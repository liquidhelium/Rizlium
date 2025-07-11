use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::{future, AsyncWriteExt};
use indexmap::IndexSet;
use rfd::AsyncFileDialog;
use rust_i18n::t;
use serde::{Deserialize, Serialize};

use crate::{ChartLoadingEvent, EditorCommands};
use helium_framework::prelude::ToastsStorage;
use rizlium_render::GameChart;

pub struct FilePlugin;

impl Plugin for FilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingDialog>()
            .init_resource::<PendingSave>()
            .add_systems(
                PostUpdate,
                (
                    open_chart,
                    report_error_or_add_current,
                    poll_pending_save.run_if(|res: Res<PendingSave>| res.0.is_some()),
                ),
            );
    }
}

#[derive(Resource, Default)]
pub struct PendingDialog(Option<Task<Option<String>>>);

pub fn open_dialog(container: &mut PendingDialog) {
    info!("opening chart");
    container.0 = Some(IoTaskPool::get().spawn(async {
        let file = AsyncFileDialog::new()
            .add_filter(t!("chart_type.bundled"), &["zip"])
            .pick_file()
            .await;

        file.map(|file| file.path().to_string_lossy().into_owned())
    }));
}

fn open_chart(mut dialog: ResMut<PendingDialog>, mut editor_command: EditorCommands) {
    if let Some(chart) = dialog
        .0
        .as_mut()
        .and_then(|t| future::block_on(future::poll_once(t)))
    {
        dialog.0.take();
        if let Some(chart) = chart {
            editor_command.load_chart(chart);
        }
    }
}

fn report_error_or_add_current(
    mut commands: Commands,
    mut events: EventReader<ChartLoadingEvent>,
    mut toasts: ResMut<ToastsStorage>,
) {
    for event in events.read() {
        match event {
            ChartLoadingEvent::Error(err) => {
                toasts.error(t!("chart.load.fail", err = err));
            }
            ChartLoadingEvent::Success(path) => {
                toasts.success(t!("chart.load.success"));
                commands.insert_resource(CurrentChartPath(path.clone()));
            }
        }
    }
}

#[derive(Resource, Deref)]
pub struct CurrentChartPath(String);

pub fn save_chart(
    chart: Option<Res<GameChart>>,
    current_path: Option<Res<CurrentChartPath>>,
    mut toasts: ResMut<ToastsStorage>,
    mut save: ResMut<PendingSave>,
) {
    let (Some(chart), Some(current_path)) = (chart, current_path) else {
        toasts.error("No chart loaded");
        return;
    };
    let path = std::path::Path::new(&**current_path);
    let Some(parent) = path.parent() else {
        toasts.error(
            "Failed while saving chart: target path have no parent \n This is probably a bug",
        );
        return;
    };
    let name = path
        .file_name()
        .map(|inner| inner.to_string_lossy())
        .unwrap_or(std::borrow::Cow::Borrowed("chart"));
    let target = parent.join(name.into_owned() + ".rzl");
    let owned_chart = (**chart).clone();
    let task: Task<Result<(), Box<dyn std::error::Error + Send + Sync>>> =
        IoTaskPool::get().spawn(async move {
            let mut file = async_fs::File::create(target).await?;
            let serialized = serde_json::to_vec(&owned_chart)?;
            file.write_all(&serialized).await?;
            file.close().await?;
            Ok(())
        });
    save.0 = Some(task);
}

#[derive(Resource, Default)]
pub struct PendingSave(Option<Task<Result<(), Box<dyn std::error::Error + Send + Sync>>>>);

fn poll_pending_save(mut save: ResMut<PendingSave>, mut toasts: ResMut<ToastsStorage>) {
    let Some(result) = save
        .0
        .as_mut()
        .and_then(|c| futures_lite::future::block_on(futures_lite::future::poll_once(c)))
    else {
        return;
    };
    save.0 = None;
    match result {
        Ok(()) => toasts.success("Chart saved!"),
        Err(err) => toasts.error(format!("error encountered while saving chart: {err}")),
    };
}

#[derive(Resource, Serialize, Deserialize, Debug, Deref)]
pub struct RecentFiles(#[deref] IndexSet<String>, usize);

impl Default for RecentFiles {
    fn default() -> Self {
        Self(default(), 4)
    }
}

impl RecentFiles {
    pub fn push(&mut self, name: String) {
        if let (idx, false) = self.0.insert_full(name) {
            let value = self.0.shift_remove_index(idx).unwrap();
            self.0.insert(value);
        }
        if self.0.len() > self.1 {
            self.0.shift_remove_index(0);
        }
    }
}

#[cfg(test)]
mod test {
    use super::RecentFiles;

    #[test]
    fn push() {
        let mut rec = RecentFiles::default();
        for i in 1..5 {
            rec.push(i.to_string());
        }
        assert_eq!(
            "RecentFiles({\"1\", \"2\", \"3\", \"4\"}, 4)".to_string(),
            format!("{:?}", rec)
        );
        rec.push("1".into());
        assert_eq!(
            "RecentFiles({\"2\", \"3\", \"4\", \"1\"}, 4)".to_string(),
            format!("{:?}", rec)
        );
        rec.push("3".into());
        assert_eq!(
            "RecentFiles({\"2\", \"4\", \"1\", \"3\"}, 4)".to_string(),
            format!("{:?}", rec)
        );
    }
}
