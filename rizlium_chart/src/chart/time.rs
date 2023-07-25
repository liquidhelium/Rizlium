use crate::chart::KeyPoint;

/// Helper function to convert real time to beat.
pub fn real2beat(start_realtime: f32, current_realtime: f32, bpm_control: &KeyPoint<f32>) -> f32 {
    start_realtime + (current_realtime - bpm_control.time) / bpm_control.value * 60.0
}