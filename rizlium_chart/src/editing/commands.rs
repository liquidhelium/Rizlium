use super::Result;
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
    MovePoint,
    RemovePoint,
    CommandSequence
}

#[enum_dispatch]
pub trait ChartCommand {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands>;
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
}
