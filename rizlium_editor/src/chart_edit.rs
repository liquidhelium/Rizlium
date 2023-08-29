use rizlium_chart::prelude::Chart;
use snafu::Snafu;

use self::{chart_path::NotePath, commands::{ChartCommands, ChartCommand}};

pub mod commands;
pub mod chart_path;

#[derive(Snafu, Debug)]
pub enum ChartConflictError {
    InvalidNotePath { note_path: NotePath },
}

type Result<T> = std::result::Result<T, ChartConflictError>;

pub struct EditHistory {
    _history_descriptions: Vec<String>,
    inverse_history: Vec<ChartCommands>,
    last_preedit_inverse: Option<ChartCommands>,
    is_preediting: bool, 
}

impl EditHistory {
    pub fn push(&mut self, edit: ChartCommands, chart: &mut Chart) -> Result<()> {
        // TODO: desc
        let inversed = edit.apply(chart)?;
        self.inverse_history.push(inversed);
        Ok(())
    }
    pub fn start_preedit(&mut self, edit: ChartCommands, chart: &mut Chart) -> Result<()> {
        if self.last_preedit_inverse.is_some() {
            panic!("Trying to start preedit when already started");
        }
        self.last_preedit_inverse = Some(edit.apply(chart)?);
        self.is_preediting = true;
        Ok(())
    }
    // pub fn push_preedit(&mut self, edit: ChartCommands, chart: &mut Chart) -> Result<()> {
    //     assert!(self.is_preediting, "Not preediting");
    //     if let Some(last) = self.last_preedit_inverse {

    //     }
    // }
}


#[cfg(test)]
mod test_resources {
    use rizlium_chart::prelude::{Chart, RizlineChart};
    use serde_json::from_str;
    const CHART_TEXT: &str = include_str!("../../assets/take.json");
    #[static_init::dynamic]
    pub static CHART: Chart = from_str::<RizlineChart>(CHART_TEXT).unwrap().try_into().unwrap();
}