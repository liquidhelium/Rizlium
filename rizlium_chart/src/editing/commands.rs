use std::{any::type_name, borrow::Cow};

use super::{ChartConflictError, Result};
use enum_dispatch::enum_dispatch;
use crate::prelude::Chart;
mod note;
pub use note::*;
mod lines;
pub use lines::*;

#[enum_dispatch(ChartCommand)]
pub enum ChartCommands {
    ChangeNoteTime,
    InsertNote,
    RemoveNote,
    InsertLine,
    RemoveLine,
    InsertPoint,
    EditPoint,
    RemovePoint,
    CommandSequence,
    Nop,
}

#[enum_dispatch]
pub trait ChartCommand {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands>;
    fn validate(&self, chart: &Chart) -> Result<()>;
    fn description(&self) -> Cow<'static, str> {
        type_name::<Self>().into()
    }
}

pub struct CommandSequence {
    commands: Vec<ChartCommands>,
}

impl ChartCommand for CommandSequence {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        Ok(Self {
            commands: self
                .commands
                .into_iter()
                // reverse to ensure inversed commands get processed in the correct order
                .rev()
                .map(|command| command.apply(chart))
                .collect::<Result<Vec<_>>>()?,
        }
        .into())
    }
    fn validate(&self,chart: &Chart) -> Result<()> {
        self.commands.iter().try_for_each(|command| command.validate(chart))
    }
}

pub struct Nop;

impl ChartCommand for Nop {
    fn apply(self,_chart: &mut Chart) -> Result<ChartCommands> {
        Ok(Nop.into())
    }
    fn validate(&self,_chart: &Chart) -> Result<()> {
        Ok(())
    }
}
