use bevy::{prelude::*, render::primitives::Aabb, DefaultPlugins};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_prototype_lyon::{
    brush::{GradientStop},
    prelude::*,
};
use mouse_tracking::{
    prelude::{InitWorldTracking},
    MainCamera,
};
use rizlium_chart::{
    __test_chart,
    chart::{RizChart},
};
use std::ops::Deref;

#[derive(Resource)]
struct GameChart(RizChart);

impl GameChart {
    pub fn get_chart(&self) -> &RizChart {
        &self.0
    }
    pub fn iter_segment(&self) -> impl Iterator<Item = (usize, usize)> + '_ {
        self.lines
            .iter()
            .enumerate()
            .map(|(i, l)| std::iter::repeat(i).zip(0..l.points.points.len() - 1))
            .flatten()
    }
}
impl Deref for GameChart {
    type Target = RizChart;
    fn deref(&self) -> &Self::Target {
        self.get_chart()
    }
}

#[derive(Component, Reflect, Debug, Default)]
#[reflect(Component)]
struct ChartLine;

pub fn start() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            ShapePlugin,
            TypeRegisterPlugin,
            line_rendering::ChartLinePlugin,
        ))
        .add_systems(Startup, before_render)
        .run();
}
pub struct TypeRegisterPlugin;
impl Plugin for TypeRegisterPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ChartLine>();
    }
}

mod line_rendering;

fn before_render(mut commands: Commands, mut window: Query<&mut Window>) {
    commands
        .spawn(Camera2dBundle {
            projection: OrthographicProjection {
                viewport_origin: [-0.5, 0.].into(),
                scaling_mode: bevy::render::camera::ScalingMode::Fixed {
                    width: 900.,
                    height: 1600.,
                },
                ..default()
            },
            ..default()
        })
        .add(InitWorldTracking)
        .insert(MainCamera);
    commands.insert_resource(GameChart(__test_chart()));
    window.single_mut().resolution.set(450., 800.);
}
