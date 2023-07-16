mod color;
mod easing;
mod line;
mod note;
use crate::Refc;

pub use color::*;
pub use easing::*;
pub use line::*;
pub use note::*;

#[derive(Debug, Clone)]
pub struct RizChart {
    pub lines: Vec<Line>,
    pub canvas: Vec<Refc<Spline<f32>>>,
    pub bpm: Spline<f32>,
}

impl RizChart {
    pub fn new(lines: Vec<Line>, canvas: Vec<Refc<Spline<f32>>>, bpm: Spline<f32>) -> Self {
        Self { lines, canvas, bpm }
    }
    pub fn lines_count(&self) -> usize {
        self.lines.len()
    }
    pub fn segment_count(&self) -> usize {
        self.lines.iter().map(|l| l.points.points.len() - 1).sum()
    }
    pub fn lines(&self) -> &Vec<Line> {
        &self.lines
    }
}

#[cfg(test)]
mod test {

    use std::fs;

    use crate::parse;

    use super::RizChart;

    #[test]
    fn test() {
        let a: RizChart = serde_json::from_str::<parse::official::RizlineChart>(include_str!(
            "../test_assets/take.json"
        ))
        .unwrap()
        .into();
        fs::write("./conv", format!("{:#?}", a)).unwrap();
    }
}
