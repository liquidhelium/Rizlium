use crate::{chart::Canvas, prelude::{Chart, KeyPoint, Line, LinePointData, Note, Spline, Tween}};

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
        let line = self.0.get(chart)?;
        if line.notes.len() > self.1 {
            Ok(())
        } else {
            Err(ChartConflictError::InvalidNotePath { note_path: *self })
        }
    }
}

impl From<(usize, usize)> for NotePath {
    fn from((i, j): (usize, usize)) -> Self {
        Self(i.into(), j)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
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
        } else {
            Err(ChartConflictError::InvalidLinePath { line_path: *self })
        }
    }
}

impl From<usize> for LinePath {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinePointPath(pub LinePath, pub usize);

impl ChartPath for LinePointPath {
    type Out = KeyPoint<f32, LinePointData>;
    fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Self::Out> {
        self.0
            .get(chart)?
            .points
            .points
            .get(self.1)
            .ok_or(ChartConflictError::NoSuchPoint {
                line_path: self.0,
                point: self.1,
            })
    }
    fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Self::Out> {
        self.0.get_mut(chart)?.points.points.get_mut(self.1).ok_or(
            ChartConflictError::NoSuchPoint {
                line_path: self.0,
                point: self.1,
            },
        )
    }
    fn remove(&self, chart: &mut Chart) -> Result<Self::Out> {
        let line = self.0.get_mut(chart)?;
        line.points
            .remove(self.1)
            .ok_or(ChartConflictError::NoSuchPoint {
                line_path: self.0,
                point: self.1,
            })
    }
    fn valid(&self, chart: &Chart) -> Result<()> {
        let line = self.0.get(chart)?;
        if line.points.len() > self.1 {
            Ok(())
        } else {
            Err(ChartConflictError::NoSuchPoint {
                line_path: self.0,
                point: self.1,
            })
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CanvasPath(pub usize);

impl ChartPath for CanvasPath {
    type Out = Canvas;
    fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Canvas> {
        chart
            .canvases
            .get(self.0)
            .ok_or(ChartConflictError::NoSuchCanvas { canvas: self.0 })
    }
    fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Canvas> {
        chart
            .canvases
            .get_mut(self.0)
            .ok_or(ChartConflictError::NoSuchCanvas { canvas: self.0 })
    }
    fn remove(&self, chart: &mut Chart) -> Result<Self::Out> {
        let len = chart.canvases.len();
        (len > self.0)
            .then(|| chart.canvases.remove(self.0))
            .ok_or(ChartConflictError::NoSuchCanvas { canvas: self.0 })
    }
    fn valid(&self, chart: &Chart) -> Result<()> {
        if chart.canvases.len() > self.0 {
            Ok(())
        } else {
            Err(ChartConflictError::NoSuchCanvas { canvas: self.0 })
        }
    }
}

impl From<usize> for CanvasPath {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

pub trait ChartSplineSelector: 'static + Copy + Clone + std::fmt::Debug + PartialEq + Eq {
    type Value: Tween + Clone + std::fmt::Debug + PartialEq + 'static;
    fn get_spline(chart: &Chart) -> &Spline<Self::Value>;
    fn get_spline_mut(chart: &mut Chart) -> &mut Spline<Self::Value>;
    fn spline_name() -> &'static str;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeControlSelector;
impl ChartSplineSelector for ThemeControlSelector {
    type Value = usize;
    fn get_spline(chart: &Chart) -> &Spline<usize> { &chart.theme_control }
    fn get_spline_mut(chart: &mut Chart) -> &mut Spline<usize> { &mut chart.theme_control }
    fn spline_name() -> &'static str { "ThemeControl" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BpmSelector;
impl ChartSplineSelector for BpmSelector {
    type Value = f32;
    fn get_spline(chart: &Chart) -> &Spline<f32> { &chart.bpm }
    fn get_spline_mut(chart: &mut Chart) -> &mut Spline<f32> { &mut chart.bpm }
    fn spline_name() -> &'static str { "Bpm" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CamScaleSelector;
impl ChartSplineSelector for CamScaleSelector {
    type Value = f32;
    fn get_spline(chart: &Chart) -> &Spline<f32> { &chart.cam_scale }
    fn get_spline_mut(chart: &mut Chart) -> &mut Spline<f32> { &mut chart.cam_scale }
    fn spline_name() -> &'static str { "CamScale" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CamMoveSelector;
impl ChartSplineSelector for CamMoveSelector {
    type Value = f32;
    fn get_spline(chart: &Chart) -> &Spline<f32> { &chart.cam_move }
    fn get_spline_mut(chart: &mut Chart) -> &mut Spline<f32> { &mut chart.cam_move }
    fn spline_name() -> &'static str { "CamMove" }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalSplinePath<S>(pub usize, pub std::marker::PhantomData<S>);

impl<S> GlobalSplinePath<S> {
    pub fn new(idx: usize) -> Self {
        Self(idx, std::marker::PhantomData)
    }
}

impl<S> From<usize> for GlobalSplinePath<S> {
    fn from(idx: usize) -> Self {
        Self::new(idx)
    }
}

impl<S: ChartSplineSelector> ChartPath for GlobalSplinePath<S> {
    type Out = KeyPoint<S::Value>;
    fn get<'c>(&self, chart: &'c Chart) -> Result<&'c Self::Out> {
        S::get_spline(chart)
            .points()
            .get(self.0)
            .ok_or(ChartConflictError::NoSuchGlobalSplinePoint {
                spline: S::spline_name(),
                index: self.0,
            })
    }
    fn get_mut<'c>(&self, chart: &'c mut Chart) -> Result<&'c mut Self::Out> {
        S::get_spline_mut(chart)
            .points
            .get_mut(self.0)
            .ok_or(ChartConflictError::NoSuchGlobalSplinePoint {
                spline: S::spline_name(),
                index: self.0,
            })
    }
    fn remove(&self, chart: &mut Chart) -> Result<Self::Out> {
        let spline = S::get_spline_mut(chart);
        if spline.points().len() > self.0 {
             Ok(spline.points.remove(self.0))
        } else {
            Err(ChartConflictError::NoSuchGlobalSplinePoint {
                spline: S::spline_name(),
                index: self.0,
            })
        }
    }
    fn valid(&self, chart: &Chart) -> Result<()> {
        if S::get_spline(chart).points().len() > self.0 {
            Ok(())
        } else {
            Err(ChartConflictError::NoSuchGlobalSplinePoint {
                spline: S::spline_name(),
                index: self.0,
            })
        }
    }
}

pub type ThemeControlPath = GlobalSplinePath<ThemeControlSelector>;
pub type BpmPath = GlobalSplinePath<BpmSelector>;
pub type CamScalePath = GlobalSplinePath<CamScaleSelector>;
pub type CamMovePath = GlobalSplinePath<CamMoveSelector>;