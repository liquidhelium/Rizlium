use bevy::render::view::RenderLayers;
use bevy_prototype_lyon::shapes::Rectangle;

use crate::GameChart;
use crate::GameTime;

use super::colorrgba_to_color;

use super::GameCamera;

use bevy::math::Vec3A;

use bevy::prelude::*;
use bevy::render::primitives::Aabb;
use bevy_prototype_lyon::prelude::*;

pub const MASK_LAYER: usize = 1;

pub struct MaskPlugin;

impl Plugin for MaskPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init_mask)
            .add_systems(PostUpdate, (update_mask_bottom, update_mask_top));
    }
}

#[derive(Component)]
pub(crate) struct MaskBottom;

#[derive(Component)]
pub(crate) struct MaskTop;

pub(crate) fn init_mask(mut commands: Commands) {
    commands
        .spawn((
            ShapeBundle {
                transform: Transform::from_xyz(0., 0., 10.),
                aabb: Aabb {
                    center: default(),
                    half_extents: Vec3A::MAX,
                },
                ..default()
            },
            Fill::default(),
            RenderLayers::layer(MASK_LAYER),
        ))
        .insert(Name::new("mask_bottom"))
        .insert(MaskBottom);
    commands
        .spawn((
            ShapeBundle {
                transform: Transform::from_xyz(0., 0., 10.),
                aabb: Aabb {
                    center: default(),
                    half_extents: Vec3A::MAX,
                },
                ..default()
            },
            Fill::default(),
            RenderLayers::layer(MASK_LAYER),
        ))
        .insert(Name::new("mask_top"))
        .insert(MaskTop);
}

pub(crate) const GRADIENT_NORMALIZED_HEIGHT: f32 = 0.05;

pub(crate) const RING_OFFSET: f32 = 0.2;

pub(crate) const TOP_MASK_HEIGHT: f32 = 0.2;

pub(crate) fn update_mask_bottom(
    mut mask_bottom: Query<(&mut Fill, &mut Path), With<MaskBottom>>,
    cams: Query<&OrthographicProjection, With<GameCamera>>,
    chart: Option<Res<GameChart>>,
    time: Res<GameTime>,
) {
    let game = cams.single();
    let area = game.area;
    let gradient_height = game.area.height() * GRADIENT_NORMALIZED_HEIGHT;
    let gradient_rect = Rect {
        min: Vec2 {
            x: area.min.x - area.width(),
            y: -gradient_height,
        },
        max: Vec2 {
            x: area.max.x + area.width(),
            y: 0.,
        },
    };
    let mask_non_transparent_rect = Rect {
        max: Vec2 {
            x: area.max.x + area.width(),
            y: -gradient_height,
        },
        min: Vec2 {
            x: area.min.x - area.width(),
            y: area.min.y - area.height(),
        },
    };
    let (mut fill, mut path) = mask_bottom.single_mut();
    fill.brush = {
        let gradient: Gradient = {
            let mut linear =
                LinearGradient::new_empty(Vec2::new(0., -gradient_height), Vec2::new(0., 0.));
            if let Some(color) = chart.and_then(|chart| {
                chart
                    .theme_at(**time)
                    .ok()
                    .map(|t| colorrgba_to_color(t.this.color.background))
            }) {
                linear.add_stop(0., color.with_alpha(1.));
                linear.add_stop(1., color.with_alpha(0.));
            }
            linear.into()
        };
        gradient.into()
    };
    *path = GeometryBuilder::new()
        .add(&Rectangle::new(mask_non_transparent_rect))
        .add(&Rectangle::new(gradient_rect))
        .build();
}

pub(crate) fn update_mask_top(
    mut mask_top: Query<(&mut Fill, &mut Path), With<MaskTop>>,
    cams: Query<&OrthographicProjection, With<GameCamera>>,
    chart: Option<Res<GameChart>>,
    time: Res<GameTime>,
) {
    let game = cams.single();
    let area = game.area;
    let gradient_height = game.area.height() * GRADIENT_NORMALIZED_HEIGHT;
    let mask_height = game.area.height() * TOP_MASK_HEIGHT;
    let gradient_rect = Rect {
        min: Vec2 {
            x: area.min.x - area.width(),
            y: area.max.y - mask_height,
        },
        max: Vec2 {
            x: area.max.x + area.width(),
            y: area.max.y - mask_height + gradient_height,
        },
    };
    let mask_non_transparent_rect = Rect {
        max: Vec2 {
            x: area.max.x + area.width(),
            y: area.max.y + area.height(),
        },
        min: Vec2 {
            x: area.min.x - area.width(),
            y: area.max.y - mask_height + gradient_height,
        },
    };
    let (mut fill, mut path) = mask_top.single_mut();
    fill.brush = {
        let gradient: Gradient = {
            let mut linear = LinearGradient::new_empty(
                Vec2::new(0., area.max.y - mask_height),
                Vec2::new(0., area.max.y - mask_height + gradient_height),
            );
            if let Some(color) = chart.and_then(|chart| {
                chart
                    .theme_at(**time)
                    .ok()
                    .map(|t| colorrgba_to_color(t.this.color.background))
            }) {
                linear.add_stop(0., color.with_alpha(0.));
                linear.add_stop(1., color.with_alpha(1.));
            }
            linear.into()
        };
        gradient.into()
    };
    *path = GeometryBuilder::new()
        .add(&Rectangle::new(mask_non_transparent_rect))
        .add(&Rectangle::new(gradient_rect))
        .build();
}
