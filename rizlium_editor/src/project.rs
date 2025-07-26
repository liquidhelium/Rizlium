use bevy::prelude::*;
use rizlium_chart::prelude::*;
use rizlium_render::ChartProvider;
use std::path::PathBuf;

pub struct ProjectPlugin;

impl Plugin for ProjectPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ProjectState>();
    }
}

#[derive(Resource, Default)]
pub enum ProjectState {
    #[default]
    Idle,
    Pending(PendingChart),
    Folder(PathBuf),
    Bundle(PathBuf, Chart),
}

pub fn has_chart(chart: Option<Res<ProjectState>>) -> bool {
    chart.is_some_and(|c| c.has_chart())
}


pub struct PendingChart {

}
pub struct FolderState {
    path: PathBuf,
    chart: Chart,
}

impl ChartProvider for ProjectState {
    fn chart(&self) -> &Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::Pending(_) => panic!("chart is pending"),
            Self::Bundle(_, c) => c,
            Self::Folder(_) => todo!()
        }
    }

    fn chart_mut(&mut self) -> &mut Chart {
        match self {
            Self::Idle => panic!("No chart loaded"),
            Self::Pending(_) => panic!("chart is pending"),
            Self::Bundle(_, c) => c,
            Self::Folder(_) => todo!()
        }
    }
    fn has_chart_system() -> impl Condition<()> {
        IntoSystem::into_system(has_chart)
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
    pub fn has_chart(&self) -> bool {
        matches!(self, Self::Folder(_) | Self::Bundle(_,_))
    }
}
