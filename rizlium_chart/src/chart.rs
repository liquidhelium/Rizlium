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
#[cfg(feature = "deserialize")]
use serde::Deserialize;
#[cfg(feature = "serialize")]
use serde::Serialize;
use snafu::{OptionExt, Whatever};
pub use theme::*;
pub use time::*;

/// Rizlium谱面格式.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
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
    pub fn note_count(&self) -> usize {
        self.lines.iter().map(|l| l.notes.len()).sum()
    }
    pub const fn with_cache<'a: 'b, 'b>(&'a self, cache: &'b ChartCache) -> ChartAndCache<'a, 'b> {
        ChartAndCache { chart: self, cache }
    }
}

/// 用于改变线形状.
///
/// 所有 [`Line`] 上的点可以附着到 [`Canvas`] 上, 并随 [`Canvas`] 移动改变位置, 从而改变线的形状.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct Canvas {
    pub x_pos: Spline<f32>,
    pub speed: Spline<f32>,
}

/// 一些可以从谱面计算出且只在对应 [`Chart`] 的数据更改时过期的数据.
#[derive(Debug, Default)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "deserialize", derive(Deserialize))]
pub struct ChartCache {
    /// 缓存的从实际时间转换为beat的数据.
    pub beat: Spline<f32>,
    pub beat_remap: Spline<f32>,
    /// 所有 [`Canvas`] 在某时间对应的高度 (从速度计算而来)
    pub canvas_y_by_real: Vec<Spline<f32>>,
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
        self.update_beat(&chart.bpm);
        self.beat_remap = self.beat.clone_reversed();
        self.canvas_y_by_real = chart
            .canvases
            .iter()
            .map(|canvas| {
                let mut points = canvas.speed.clone().with_relevant::<f32>().points;
                points.push(KeyPoint {
                    time: points.last().unwrap().time + LARGE,
                    value: 0.,
                    ease_type: EasingId::Start,
                    relevent: 0.,
                });
                points.iter_mut().fold(
                    (0., 0., 0.0f32),
                    |(last_start, last_real, last_value), keypoint| {
                        let this_real = self.beat_remap.value_padding(keypoint.time).unwrap();
                        let pos = last_value.mul_add(this_real - last_real, last_start);
                        let value = keypoint.value;
                        keypoint.value = pos;
                        keypoint.time = this_real;
                        (pos, this_real, value)
                    },
                );

                points
                    .into_iter()
                    .map(|k| KeyPoint {
                        time: k.time,
                        value: k.value,
                        ease_type: k.ease_type,
                        relevent: (),
                    })
                    .collect()
            })
            .collect();
    }

    pub fn canvas_y_at(&self, index: usize, time: f32) -> Option<f32> {
        let canvas = self.canvas_y_by_real.get(index)?;
        let real_time = self.beat_remap.value_padding(time).unwrap();
        canvas.value_padding(real_time)
    }

    pub(crate) fn update_beat(&mut self, spline: &Spline<f32>) {
        let last = KeyPoint {
            time: spline.points().last().unwrap().time + LARGE,
            value: 0.,
            ease_type: EasingId::Start,
            relevent: (),
        };
        let mut iter = spline.iter().chain(Some(&last));
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
    }
    pub fn map_time(&self, time: f32) -> f32 {
        self.beat.value_padding(time).expect("empty beat spline")
    }
    pub fn remap_beat(&self, game_time: f32) -> f32 {
        self.beat_remap
            .value_padding(game_time)
            .expect("empty beat spline (remap)")
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
        Some([
            self.keypoint_releated_x(point, game_time)?,
            self.cache.canvas_y_at(point.relevent, point.time)?
                - self.cache.canvas_y_at(point.relevent, game_time)?,
        ])
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
        let point_y = self.cache.canvas_y_at(point1.relevent, time)?
        - self.cache.canvas_y_at(point1.relevent, game_time)?;
        Some([
            f32::ease(
                pos1[0],
                pos2[0],
                invlerp(pos1[1], pos2[1], point_y),
                point1.ease_type,
            ),
            point_y
        ])
    }
    pub fn line_pos_at_clamped(
        &self,
        line_idx: usize,
        mut time: f32,
        game_time: f32,
    ) -> Option<[f32; 2]> {
        let line = self.chart.lines.get(line_idx)?;
        time = time.clamp(
            line.points.start_time()? + 0.01,
            line.points.end_time()? - 0.01,
        );
        self.line_pos_at(line_idx, time, game_time)
    }

    fn keypoint_releated_x(&self, point: &KeyPoint<f32, usize>, time: f32) -> Option<f32> {
        Some(point.value + self.chart.canvas_x(point.relevent, time)?)
    }
    pub fn has_speed_mutation(&self, line_index: usize, segment_start: usize) -> Option<bool> {
        let (this, next) = self.chart.lines.get(line_index).and_then(|l| {
            l.points
                .points
                .get(segment_start)
                .zip(l.points.points.get(segment_start + 1))
        })?;
        if this.relevent != next.relevent {
            Some(false)
        } else {
            let canvas = self.cache.canvas_y_by_real.get(this.relevent)?;
            if canvas.keypoint_at(this.time) != canvas.keypoint_at(next.time) {
                use log::info;
                info!("return true");
                Some(true)
            } else {
                Some(false)
            }
        }
    }
}
