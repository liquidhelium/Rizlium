use bevy::{
    prelude::*,
    tasks::{IoTaskPool, Task},
};
use futures_lite::future;
use indexmap::IndexSet;
use rfd::AsyncFileDialog;
use serde::{Deserialize, Serialize};

use crate::EditorCommands;

pub struct FilePlugin;

impl Plugin for FilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingDialog>()
            .add_systems(PostUpdate, open_chart);
    }
}

#[derive(Resource, Default)]
pub struct PendingDialog(Option<Task<Option<String>>>);

pub fn open_dialog(container: &mut PendingDialog) {
    info!("opening chart");
    container.0 = Some(IoTaskPool::get().spawn(async {
        let file = AsyncFileDialog::new()
            .add_filter("Bundled chart file", &["zip"])
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
