use bevy::prelude::*;
use rizlium_chart::chart::Chart;

#[derive(Resource, Reflect, Clone, Debug)]
#[reflect(Resource)]
pub struct SnappingConfig {
    pub enable_time_snap: bool,
    pub time_divisor: u32,
    pub enable_value_snap: bool,
    pub value_step: f32,
}

impl Default for SnappingConfig {
    fn default() -> Self {
        Self {
            enable_time_snap: true,
            time_divisor: 4,
            enable_value_snap: true,
            value_step: 100.0,
        }
    }
}

pub struct SnappingContext<'a> {
    pub config: &'a SnappingConfig,
    #[allow(dead_code)]
    pub chart: Option<&'a Chart>,
}

impl<'a> SnappingContext<'a> {
    pub fn new(config: &'a SnappingConfig, chart: Option<&'a Chart>) -> Self {
        Self { config, chart }
    }

    pub fn snap_time(&self, time: f32) -> f32 {
        if !self.config.enable_time_snap || self.config.time_divisor == 0 {
            return time;
        }
        
        let step = 1.0 / self.config.time_divisor as f32;
        (time / step).round() * step
    }

    pub fn snap_value(&self, value: f32) -> f32 {
        if !self.config.enable_value_snap || self.config.value_step <= 0.0 {
            return value;
        }
        
        (value / self.config.value_step).round() * self.config.value_step
    }
    
    pub fn snap_point(&self, time: f32, value: f32) -> (f32, f32) {
        (self.snap_time(time), self.snap_value(value))
    }
}
