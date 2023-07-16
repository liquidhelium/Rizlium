use super::*;
#[derive(Debug, Clone)]
pub struct Line {
    /// Related to canvas
    pub points: Spline<f32>,
    /// Related to `line_color`
    pub point_color: Spline<ColorRGBA>,
    pub notes: Vec<Note>,
    pub ring_color: Spline<ColorRGBA>,
    pub vertical_move: Refc<Spline<f32>>,
    pub line_color: Refc<Spline<ColorRGBA>>,
}

impl Line {
    pub fn new(
        points: Spline<f32>,
        point_color: Spline<ColorRGBA>,
        notes: Vec<Note>,
        ring_color: Spline<ColorRGBA>,
        vertical_move: Refc<Spline<f32>>,
        line_color: Refc<Spline<ColorRGBA>>,
    ) -> Self {
        Self {
            points,
            point_color,
            notes,
            ring_color,
            vertical_move,
            line_color,
        }
    }
    pub fn pos_for(&self, point_idx: usize, game_time: f32) -> Option<[f32; 2]> {
        let key_point = self.points.points.get(point_idx)?;
        Some([
            key_point.related_value(game_time),
            self.vertical_move.value_at(key_point.time),
        ])
    }
    pub fn pos_at_time(&self, time: f32, game_time: f32) -> [f32; 2] {
        [
            self.points.value_at_related(time, game_time),
            self.vertical_move.value_at(time),
        ]
    }
    pub fn try_pos_at_time(&self, time: f32, game_time: f32) -> Option<[f32; 2]> {
        Some([
            self.points.try_value_at_related(time, game_time)?,
            self.vertical_move.value_at(time),
        ])
    }
}
