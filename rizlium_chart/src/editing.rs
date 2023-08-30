use crate::prelude::Chart;
use snafu::Snafu;

use self::{
    chart_path::NotePath,
    commands::{ChartCommand, ChartCommands},
};
/// Representation of a chart item
pub mod chart_path;
pub mod commands;

#[derive(Snafu, Debug)]
pub enum ChartConflictError {
    InvalidNotePath { note_path: NotePath },
}

type Result<T> = std::result::Result<T, ChartConflictError>;

pub struct EditHistory {
    _history_descriptions: Vec<String>,
    inverse_history: Vec<ChartCommands>,
    last_preedit_inverse: Option<ChartCommands>,
}

impl EditHistory {
    pub fn push(&mut self, edit: ChartCommands, chart: &mut Chart) -> Result<()> {
        // TODO: desc
        let inversed = edit.apply(chart)?;
        self.inverse_history.push(inversed);
        Ok(())
    }
    pub fn push_preedit(&mut self, edit: ChartCommands, chart: &mut Chart) -> Result<()> {
        let current_inverse = edit.apply(chart)?;
        if let Some(last) = self.last_preedit_inverse.take() {
            last.apply(chart)?;
        }
        self.last_preedit_inverse = Some(current_inverse);
        Ok(())
    }
}

#[cfg(test)]
mod test_resources {
    use crate::prelude::{Chart, RizlineChart};
    use serde_json::from_str;
    const CHART_TEXT: &str = include_str!("../../assets/take.json");
    #[static_init::dynamic]
    pub static CHART: Chart = from_str::<RizlineChart>(CHART_TEXT)
        .unwrap()
        .try_into()
        .unwrap();
}
