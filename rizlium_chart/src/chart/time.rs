use crate::chart::KeyPoint;

/// 将实际时间转换为beat.
pub fn real2beat(beat_till_last: f32, current_realtime: f32, bpm_control: &KeyPoint<f32>) -> f32 {
    ((current_realtime - bpm_control.time) / bpm_control.value).mul_add(60.0, beat_till_last)
}

/// 将beat转换为实际时间.
pub fn beat2real(beat_till_last: f32, target_beat: f32, bpm_control: &KeyPoint<f32>) -> f32 {
    ((target_beat - beat_till_last) * bpm_control.value) / 60.0 + bpm_control.time
}
