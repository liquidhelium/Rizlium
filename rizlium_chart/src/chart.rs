mod color;
mod easing;
mod line;
mod note;
mod time;

pub use color::*;
pub use easing::*;
pub use line::*;
pub use note::*;
pub use time::*;

/// Main Rizlium chart structure.
#[derive(Debug, Clone)]
pub struct Chart {
    pub lines: Vec<Line>,
    pub canvases: Vec<Canvas>,
    pub bpm: Spline<f32>,
    pub cam_scale: Spline<f32>,
    pub cam_move: Spline<f32>
}

/// A object that points of [`Line`]s can be attached to.
#[derive(Debug, Clone)]
pub struct Canvas {
    pub x_pos: Spline<f32>,
    pub speed: Spline<f32>,
}

/// Some data that can be computed from [`Chart`] and don't expire even time changed.
/// 
/// Note that when the corresponding [`Chart`] is changed, this one expires.
#[derive(Debug, Default)]
pub struct ChartCache {
    pub beat: Spline<f32>,
    pub canvas_y: Vec<Spline<f32>>
}

impl ChartCache {
    /// Create a new [`ChartCache`] from an existing [`Chart`].
    pub fn from_chart(chart: &Chart) -> Self {
        let mut ret: Self = Default::default();
        ret.update_from_chart(chart);
        ret
    }
    /// Update this [`ChartCache`] using the given [`Chart`].
    pub fn update_from_chart(&mut self,chart: &Chart) {
        let mut points = chart.bpm.points().clone();
        points.iter_mut().fold(0., |current_start, keypoint| {
            let beat = real2beat(current_start, keypoint.time, &keypoint);
            keypoint.value = beat;
            beat
        });
        
        self.beat = points.into();
        self.canvas_y = chart.canvases.iter().map(|canvas| {
            let mut points = canvas.speed.points().clone();
            points.iter_mut().fold((0., 0.), |(last_start, last_time), keypoint| {
                let pos = last_start + keypoint.value * (keypoint.time - last_time);
                keypoint.value = pos;
                (pos, keypoint.time)
            });
            points.into()
        } ).collect();
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
        .try_into().unwrap();
        fs::write("./conv", format!("{:#?}", a)).unwrap();
    }
}
