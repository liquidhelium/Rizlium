use rizlium_chart::prelude::{Chart, Note};

use super::{ChartConflictError, Result};


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotePath(pub usize, pub usize);

impl NotePath {
    pub fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Note> {
        chart
            .lines
            .get(self.0)
            .and_then(|line| line.notes.get(self.1))
            .ok_or(ChartConflictError::InvalidNotePath {
                note_path: *self
            })
    }
    pub fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Note> {
        chart
            .lines
            .get_mut(self.0)
            .and_then(|line| line.notes.get_mut(self.1))
            .ok_or(ChartConflictError::InvalidNotePath {
                note_path: *self
            })
    }
    pub fn valid(&self, chart: &Chart) -> bool {
        chart.lines.get(self.0).is_some_and(|line| line.notes.len() > self.1)
    }
}

impl From<(usize,usize)> for NotePath {
    fn from((i,j): (usize,usize)) -> Self {
        Self(i, j)
    }
}
