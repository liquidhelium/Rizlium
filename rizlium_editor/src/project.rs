use bevy::prelude::*;
use rizlium_chart::prelude::*;
use rizlium_render::ChartProvider;
use std::path::PathBuf;

#[derive(Resource, Default)]
pub enum ProjectState {
    #[default]
    Idle,
    Folder(PathBuf),
    Bundle(PathBuf, Chart),
}
pub struct FolderState {
    path: PathBuf,
    chart: Chart,
}

impl ChartProvider for ProjectState {
    fn chart(&self) -> &Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::Bundle(_, c) => c,
            Self::Folder(_) => todo!()
        }
    }

    fn chart_mut(&mut self) -> &mut Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::Bundle(_, c) => c,
            Self::Folder(_) => todo!()
        }
    }
}

impl ProjectState {
    pub fn segment_count(&self) -> usize {
        self.chart()
            .lines
            .iter()
            .map(|line| line.points.len() - 1)
            .reduce(|a, b| a + b).unwrap_or_default()
    }
    pub fn note_count(&self) -> usize {
        self.iter_note().count()
    }
}
