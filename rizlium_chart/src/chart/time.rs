use crate::chart::KeyPointNext;

pub fn real2beat(start_realtime: f32, current_realtime: f32, bpm_control: &KeyPointNext<f32>) -> f32 {
    start_realtime + (current_realtime - bpm_control.time) / bpm_control.value * 60.0
}