use std::marker::PhantomData;
use std::mem::replace;

use crate::{
    chart::{EasingId, KeyPoint},
    editing::{
        chart_path::{
            BpmSelector, CamMoveSelector, CamScaleSelector, ChartPath, ChartSplineSelector,
            GlobalSplinePath, ThemeControlSelector,
        },
        ChartConflictError, Result,
    },
    prelude::Chart,
};

use super::{ChartCommand, ChartCommands};

pub trait GlobalSplineCommandWrapper: ChartSplineSelector {
    fn wrap_insert(cmd: InsertGlobalPoint<Self>) -> ChartCommands;
    fn wrap_remove(cmd: RemoveGlobalPoint<Self>) -> ChartCommands;
    fn wrap_edit(cmd: EditGlobalPoint<Self>) -> ChartCommands;
}

#[derive(Debug)]
pub struct InsertGlobalPoint<S: ChartSplineSelector> {
    pub point: KeyPoint<S::Value>,
    pub index: Option<usize>,
    pub _phantom: PhantomData<S>,
}

impl<S: ChartSplineSelector> InsertGlobalPoint<S> {
    pub fn new(point: KeyPoint<S::Value>, index: Option<usize>) -> Self {
        Self {
            point,
            index,
            _phantom: PhantomData,
        }
    }
}

impl<S: ChartSplineSelector + GlobalSplineCommandWrapper> ChartCommand for InsertGlobalPoint<S> {
    fn apply(mut self, chart: &mut Chart) -> Result<ChartCommands> {
        let spline = S::get_spline_mut(chart);
        let points = &mut spline.points;
        let at = self.index.unwrap_or(points.len()).clamp(0, points.len());
        
        let prev_time = if at > 0 { points[at - 1].time } else { f32::NEG_INFINITY };
        let next_time = if at < points.len() { points[at].time } else { f32::INFINITY };
        
        self.point.time = self.point.time.clamp(prev_time, next_time);
        
        points.insert(at, self.point);
        
        Ok(S::wrap_remove(RemoveGlobalPoint {
            path: GlobalSplinePath::new(at),
        }))
    }

    fn validate(&self, _chart: &Chart) -> Result<()> {
        Ok(())
    }
    fn description(&self) -> std::borrow::Cow<'static,str> {
        format!(
            "Insert {} Point at time {}",
            S::spline_name(),
            self.point.time
        )
        .into()
    }
}

#[derive(Debug)]
pub struct RemoveGlobalPoint<S: ChartSplineSelector> {
    pub path: GlobalSplinePath<S>,
}

impl<S: ChartSplineSelector + GlobalSplineCommandWrapper> ChartCommand for RemoveGlobalPoint<S> {
    fn apply(self, chart: &mut Chart) -> Result<ChartCommands> {
        let point = self.path.remove(chart)?;
        Ok(S::wrap_insert(InsertGlobalPoint::new(point, Some(self.path.0))))
    }

    fn validate(&self, chart: &Chart) -> Result<()> {
        self.path.valid(chart)
    }
    fn description(&self) -> std::borrow::Cow<'static,str> {
        format!(
            "Remove {} Point at index {}",
            S::spline_name(),
            self.path.0
        )
        .into()
    }
}

#[derive(Debug)]
pub struct EditGlobalPoint<S: ChartSplineSelector> {
    pub path: GlobalSplinePath<S>,
    pub new_time: Option<f32>,
    pub new_value: Option<S::Value>,
    pub new_easing: Option<EasingId>,
}

impl<S: ChartSplineSelector + GlobalSplineCommandWrapper> ChartCommand for EditGlobalPoint<S> {
    fn apply(mut self, chart: &mut Chart) -> Result<ChartCommands> {
        let spline = S::get_spline_mut(chart);
        let points = &mut spline.points;
        
        if self.path.0 >= points.len() {
             return Err(ChartConflictError::NoSuchGlobalSplinePoint {
                spline: S::spline_name(),
                index: self.path.0,
            });
        }

        let prev_time = if self.path.0 > 0 { points[self.path.0 - 1].time } else { f32::NEG_INFINITY };
        let next_time = if self.path.0 + 1 < points.len() { points[self.path.0 + 1].time } else { f32::INFINITY };

        let point = &mut points[self.path.0];

        self.new_time = self.new_time.map(|t| t.clamp(prev_time, next_time));
        
        let old_time = self.new_time.map(|t| replace(&mut point.time, t));
        let old_value = self.new_value.map(|v| replace(&mut point.value, v));
        let old_easing = self.new_easing.map(|e| replace(&mut point.ease_type, e));

        Ok(S::wrap_edit(Self {
            path: self.path,
            new_time: old_time,
            new_value: old_value,
            new_easing: old_easing,
        }))
    }

    fn validate(&self, chart: &Chart) -> Result<()> {
        self.path.valid(chart)
    }
    fn description(&self) -> std::borrow::Cow<'static,str> {
        format!(
            "Edit {} Point at index {}",
            S::spline_name(),
            self.path.0
        )
        .into()
    }
}

macro_rules! impl_wrapper {
    ($selector:ty, $insert:ident, $remove:ident, $edit:ident) => {
        impl GlobalSplineCommandWrapper for $selector {
            fn wrap_insert(cmd: InsertGlobalPoint<Self>) -> ChartCommands {
                ChartCommands::$insert(cmd)
            }
            fn wrap_remove(cmd: RemoveGlobalPoint<Self>) -> ChartCommands {
                ChartCommands::$remove(cmd)
            }
            fn wrap_edit(cmd: EditGlobalPoint<Self>) -> ChartCommands {
                ChartCommands::$edit(cmd)
            }
        }
    };
}

impl_wrapper!(ThemeControlSelector, InsertThemePoint, RemoveThemePoint, EditThemePoint);
impl_wrapper!(BpmSelector, InsertBpmPoint, RemoveBpmPoint, EditBpmPoint);
impl_wrapper!(CamScaleSelector, InsertCamScalePoint, RemoveCamScalePoint, EditCamScalePoint);
impl_wrapper!(CamMoveSelector, InsertCamMovePoint, RemoveCamMovePoint, EditCamMovePoint);
