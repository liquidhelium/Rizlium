mod color;
mod easing;
mod line;
mod note;
mod theme;
mod time;

pub use color::*;
pub use easing::*;
pub use line::*;
pub use note::*;
use snafu::{OptionExt, Whatever};
pub use theme::*;
pub use time::*;

/// Rizlium谱面格式.
#[derive(Debug, Clone)]
pub struct Chart {
    pub themes: Vec<ThemeData>,
    pub theme_control: Spline<usize>,
    pub lines: Vec<Line>,
    pub canvases: Vec<Canvas>,
    pub bpm: Spline<f32>,
    pub cam_scale: Spline<f32>,
    pub cam_move: Spline<f32>,
}

impl Chart {
    pub fn theme_at(&self, time: f32) -> Result<ThemeTransition<'_>, Whatever> {
        let (this, next) = self.theme_control.pair(time);
        let this = this.unwrap_or(
            self.theme_control
                .points()
                .first()
                .whatever_context("Empty theme control")?,
        );
        let next = next.unwrap_or(
            self.theme_control
                .points()
                .last()
                .whatever_context("Empty theme control")?,
        );
        let progress = (time - this.time) / (next.time - this.time);
        Ok(ThemeTransition {
            progress,
            this: self
                .themes
                .get(this.value)
                .whatever_context("Theme control value out of bounds")?,
            next: self
                .themes
                .get(next.value)
                .whatever_context("Theme control value out of bounds")?,
        })
    }
    pub fn canvas_x(&self, index: usize, time: f32) -> Option<f32> {
        self.canvases.get(index)?.x_pos.value_padding(time)
    }
    pub fn segment_count(&self) -> usize {
        self.lines.iter().map(|l| l.points.len() - 1).sum()
    }
    pub fn with_cache<'a: 'b, 'b>(&'a self, cache: &'b ChartCache) -> ChartAndCache<'a, 'b> {
        ChartAndCache {
            chart: &self,
            cache: &cache,
        }
    }
}

/// 用于改变线形状.
///
/// 所有 [`Line`] 上的点可以附着到 [`Canvas`] 上, 并随 [`Canvas`] 移动改变位置, 从而改变线的形状.
#[derive(Debug, Clone)]
pub struct Canvas {
    pub x_pos: Spline<f32>,
    pub speed: Spline<f32>,
}

/// 一些可以从谱面计算出且只在对应 [`Chart`] 的数据更改时过期的数据.
#[derive(Debug, Default)]
pub struct ChartCache {
    /// 缓存的从实际时间转换为beat的数据.
    pub beat: Spline<f32>,
    /// 所有 [`Canvas`] 在某时间对应的高度 (从速度计算而来)
    pub canvas_y: Vec<Spline<f32>>,
    pub canvas_y_remap: Vec<Spline<f32>>,
}

const LARGE: f32 = 1.0e10;

impl ChartCache {
    /// 从已有的 [`Chart`] 生成新 [`ChartCache`].
    pub fn from_chart(chart: &Chart) -> Self {
        let mut ret: Self = Default::default();
        ret.update_from_chart(chart);
        ret
    }
    /// 用给定的 [`Chart`] 更新此 [`ChartCache`] .
    pub fn update_from_chart(&mut self, chart: &Chart) {
        let mut iter = chart.bpm.iter();
        let mut last_time = 0.;
        let mut last_key = None;
        self.beat = std::iter::from_fn(|| {
            let point = iter.next()?;
            let Some(last) = last_key else {
                last_key = Some(point);
                return Some(KeyPoint {
                                    time:0.,
                                    value:0.0f32,
                                    ease_type: EasingId::Linear,
                                    relevent: ()
                                })
            } ;
            last_key = Some(point);
            Some(KeyPoint {
                time: {
                    let beat = real2beat(last_time, point.time, last);
                    last_time = beat;
                    beat
                },
                value: point.time,
                ease_type: EasingId::Linear,
                relevent: (),
            })
        })
        .collect();
        // self.beat.push(KeyPoint {
        //     time: LARGE,
        //     value: self.beat.points().last().unwrap().value + LARGE * chart.bpm.points().last().unwrap().value,
        //     ease_type: EasingId::Linear,
        //     relevent: (),
        // });
        self.canvas_y = chart
            .canvases
            .iter()
            .map(|canvas| {
                let mut points = canvas.speed.points().clone();
                points
                    .iter_mut()
                    .fold((0., 0.), |(last_start, last_time), keypoint| {
                        let pos = last_start + keypoint.value * (keypoint.time - last_time);
                        keypoint.value = pos;
                        (pos, keypoint.time)
                    });
                let mut  spline: Spline<_> = points.into();
                    spline.push(KeyPoint {
                        time: LARGE,
                        value: spline.points().last().unwrap().value + LARGE*canvas.speed.points()[0].value,
                        ease_type: EasingId::Linear,
                        relevent: (),
                    });
                spline
            })
            .collect();
        self.canvas_y_remap = self.canvas_y.iter().map(|i| i.clone_reversed()).collect();
    }
}

pub struct ChartAndCache<'chart, 'cache> {
    chart: &'chart Chart,
    cache: &'cache ChartCache,
}

impl ChartAndCache<'_, '_> {
    pub fn pos_for_linepoint_at(
        &self,
        line_idx: usize,
        point_idx: usize,
        game_time: f32,
    ) -> Option<[f32; 2]> {
        let point = self
            .chart
            .lines
            .get(line_idx)?
            .points
            .points()
            .get(point_idx)?;
        Some([self.keypoint_releated_x(point, game_time)?, {
            let line = self.cache.canvas_y.get(point.relevent)?;
            line.value_padding(point.time)? - line.value_padding(game_time)?
        }])
    }
    pub fn line_pos_at(&self, line_idx: usize, time: f32, game_time: f32) -> Option<[f32; 2]> {
        let line = self.chart.lines.get(line_idx)?;
        let index = line.points.keypoint_at(time).ok()?;
        let point1 = &line.points.points()[index];
        let point2 = &line.points.points()[index + 1];
        // Safe because `keypoint_at`.
        let pos1 = self
            .pos_for_linepoint_at(line_idx, index, game_time)
            .unwrap();
        let pos2 = self
            .pos_for_linepoint_at(line_idx, index + 1, game_time)
            .unwrap();
        Some([
            f32::ease(
                pos1[0],
                pos2[0],
                invlerp(point1.time, point2.time, time),
                point1.ease_type,
            ),
            f32::lerp(pos1[1], pos2[1], invlerp(point1.time, point2.time, time)),
        ])
    }

    fn keypoint_releated_x(&self, point: &KeyPoint<f32, usize>, time: f32) -> Option<f32> {
        Some(point.value + self.chart.canvas_x(point.relevent, time)?)
    }
}

#[cfg(test)]
mod test {

    use std::fs;

    use crate::parse;

    use super::Chart;

    #[test]
    fn test() {
        let a: Chart = serde_json::from_str::<parse::official::RizlineChart>(include_str!(
            "../test_assets/take.json"
        ))
        .unwrap()
        .try_into()
        .unwrap();
        fs::write("./conv", format!("{:#?}", a)).unwrap();
    }
}
