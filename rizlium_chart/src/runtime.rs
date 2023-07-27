use crate::chart::{Chart, ChartCache, ThemeTransition};

pub struct RuntimeChart<'a> {
    pub current_theme: ThemeTransition<'a>,
    pub canvas_x: Vec<f32>,
}

impl<'a> RuntimeChart<'a> {
    pub fn from_chart(chart: &'a Chart, cache: &ChartCache, time: f32) -> Self {
        // 错误处理留给后人.jpg()
        assert_eq!(chart.canvases.len(), cache.canvas_y.len(), "chart do not match cache");
        let theme = chart.theme_at(time).unwrap();
        Self {
            current_theme: theme,
            canvas_x: chart
                .canvases
                .iter()
                .map(|c| c.x_pos.value_padding(time).unwrap()).collect(),
        }
    }
}
