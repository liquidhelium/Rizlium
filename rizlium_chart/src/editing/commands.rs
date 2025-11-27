use std::{any::type_name, borrow::Cow, fmt::Debug};

use super::Result;
use crate::editing::chart_path::*;
use crate::prelude::Chart;
use enum_dispatch::enum_dispatch;
mod note;
pub use note::*;
mod lines;
pub use lines::*;
mod canvases;
pub use canvases::*;
mod global_spline;
pub use global_spline::*;

#[enum_dispatch(ChartCommand)]
#[derive(Debug, Clone)]
pub enum ChartCommands {
    ChangeNoteTime,
    InsertNote,
    RemoveNote,
    InsertLine,
    RemoveLine,
    InsertPoint,
    EditPoint,
    RemovePoint,
    InsertCanvas,
    RemoveCanvas,

    InsertThemePoint(InsertGlobalPoint<ThemeControlSelector>),
    RemoveThemePoint(RemoveGlobalPoint<ThemeControlSelector>),
    EditThemePoint(EditGlobalPoint<ThemeControlSelector>),

    InsertBpmPoint(InsertGlobalPoint<BpmSelector>),
    RemoveBpmPoint(RemoveGlobalPoint<BpmSelector>),
    EditBpmPoint(EditGlobalPoint<BpmSelector>),

    InsertCamScalePoint(InsertGlobalPoint<CamScaleSelector>),
    RemoveCamScalePoint(RemoveGlobalPoint<CamScaleSelector>),
    EditCamScalePoint(EditGlobalPoint<CamScaleSelector>),

    InsertCamMovePoint(InsertGlobalPoint<CamMoveSelector>),
    RemoveCamMovePoint(RemoveGlobalPoint<CamMoveSelector>),
    EditCamMovePoint(EditGlobalPoint<CamMoveSelector>),

    CommandSequence,
    Nop,
}

#[enum_dispatch]
pub trait ChartCommand: Debug {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands>;
    fn validate(&self, chart: &Chart) -> Result<()>;
    fn description(&self) -> Cow<'static, str> {
        type_name::<Self>().into()
    }
}

#[derive(Debug, Clone)]
pub struct CommandSequence {
    pub commands: Vec<ChartCommands>,
    pub description: Cow<'static, str>,
}

impl ChartCommand for CommandSequence {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        Ok(Self {
            commands: {
                let mut reversed_commands = self
                    .commands
                    .into_iter()
                    .map(|command| command.apply(chart))
                    .collect::<Result<Vec<_>>>()?;
                // reverse to ensure inversed commands get processed in the correct order
                // eg. if we first insert A then B, to undo we must first remove B then A
                reversed_commands.reverse();
                reversed_commands
            },
            description: self.description,
        }
        .into())
    }
    fn validate(&self, chart: &Chart) -> Result<()> {
        // we can't just validate the whole sequence at once, as earlier commands may affect later ones
        // this is less efficient, but necessary
        let mut temp_chart = chart.clone();
        for command in &self.commands {
            command.validate(&temp_chart)?;
            command.clone().apply(&mut temp_chart)?; // we can ignore the returned inverse commands here
        }
        Ok(())
    }
    fn description(&self) -> Cow<'static, str> {
        self.description.clone()
    }
}

#[derive(Debug, Clone)]
pub struct Nop;

impl ChartCommand for Nop {
    fn apply(self, _chart: &mut Chart) -> Result<ChartCommands> {
        Ok(Nop.into())
    }
    fn validate(&self, _chart: &Chart) -> Result<()> {
        Ok(())
    }
}
