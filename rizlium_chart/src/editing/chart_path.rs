use crate::prelude::{Chart, Line, Note};

use super::{ChartConflictError, Result};

pub trait ChartPath {
    type Out;
    fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Self::Out>;
    fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Self::Out>;
    fn remove(&self, chart: &mut Chart) -> Result<Self::Out>;
    fn valid(&self, chart: &Chart) -> Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotePath(pub LinePath, pub usize);

impl NotePath {
    pub fn new(line: usize, note: usize) -> Self {
        (line, note).into()
    }
}

impl ChartPath for NotePath {
    type Out = Note;
    fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Note> {
        self.0
            .get(chart)?
            .notes
            .get(self.1)
            .ok_or(ChartConflictError::InvalidNotePath { note_path: *self })
    }
    fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Note> {
        self.0
            .get_mut(chart)?
            .notes
            .get_mut(self.1)
            .ok_or(ChartConflictError::InvalidNotePath { note_path: *self })
    }
    fn remove(&self, chart: &mut Chart) -> Result<Self::Out> {
        let line = self.0.get_mut(chart)?;
        let len = line.notes.len();
        (len > self.1)
            .then(|| line.notes.remove(self.1))
            .ok_or(ChartConflictError::InvalidNotePath { note_path: *self })
    }
    fn valid(&self, chart: &Chart) -> Result<()> {
        let line = self.0
            .get(chart)?;
        if line.notes.len() > self.1 {
            Ok(())
        }
        else {
            Err(ChartConflictError::InvalidNotePath { note_path: *self })
        }
    }
}

impl From<(usize, usize)> for NotePath {
    fn from((i, j): (usize, usize)) -> Self {
        Self(i.into(), j)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinePath(pub usize);

impl ChartPath for LinePath {
    type Out = Line;
    fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Line> {
        chart
            .lines
            .get(self.0)
            .ok_or(ChartConflictError::InvalidLinePath { line_path: *self })
    }
    fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Line> {
        chart
            .lines
            .get_mut(self.0)
            .ok_or(ChartConflictError::InvalidLinePath { line_path: *self })
    }
    fn remove(&self, chart: &mut Chart) -> Result<Self::Out> {
        let len = chart.lines.len();
        (len > self.0)
            .then(|| chart.lines.remove(self.0))
            .ok_or(ChartConflictError::InvalidLinePath { line_path: *self })
    }
    fn valid(&self, chart: &Chart) -> Result<()> {
        if chart.lines.len() > self.0 {
            Ok(())
        }
        else {
            Err(ChartConflictError::InvalidLinePath { line_path: *self })
        }
    }
}

impl From<usize> for LinePath {
    fn from(value: usize) -> Self {
        Self(value)
    }
}