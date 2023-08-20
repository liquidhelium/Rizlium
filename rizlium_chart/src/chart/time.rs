use crate::chart::KeyPoint;

/// 将实际时间转换为beat.
pub fn real2beat(beat_till_last: f32, current_realtime: f32, bpm_control: &KeyPoint<f32>) -> f32 {
    ((current_realtime - bpm_control.time) / bpm_control.value).mul_add(60.0, beat_till_last)
}
