mod color;
mod easing;
mod line;
mod note;
mod time;
use crate::Refc;

pub use color::*;
pub use easing::*;
pub use line::*;
pub use note::*;
pub use time::*;

#[derive(Debug, Clone)]
pub struct ChartNext {
    pub lines: Vec<LineNext>,
    pub canvases: Vec<Canvas>,
    pub bpm: SplineNext<f32>,
    pub cam_scale: SplineNext<f32>,
    pub cam_move: SplineNext<f32>
}
#[derive(Debug, Clone)]
pub struct Canvas {
    pub x_pos: SplineNext<f32>,
    pub speed: SplineNext<f32>,
}

#[derive(Debug, Default)]
pub struct ChartCache {
    pub beat: SplineNext<f32>,
    pub canvas_y: Vec<SplineNext<f32>>
}

impl ChartCache {
    pub fn from_chart(chart: &ChartNext) -> Self {
        let mut ret: Self = Default::default();
        ret.update_from_chart(chart);
        ret
    }
    pub fn update_from_chart(&mut self,chart: &ChartNext) {
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

    use super::ChartNext;

    #[test]
    fn test() {
        let a: ChartNext = serde_json::from_str::<parse::official_next::RizlineChart>(include_str!(
            "../test_assets/take.json"
        ))
        .unwrap()
        .into();
        fs::write("./conv", format!("{:#?}", a)).unwrap();
    }
}
