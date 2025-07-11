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
pub const MASK_Z: f32 = 10.;

pub struct MaskPlugin;

impl Plugin for MaskPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GeneratedMaskTheme>()
            .add_systems(Startup, init_mask)
            .add_systems(
                PostUpdate,
                update_mask.run_if(chart_update!()),
            );
    }
}

#[derive(Resource, Default)]
struct GeneratedMaskTheme(Option<usize>);
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
                transform: Transform::from_xyz(0., 0., MASK_Z),
                aabb: Aabb {
                    center: default(),
                    half_extents: Vec3A::MAX,
                },
                ..default()
            },
            Fill::brush(Color::BLACK.with_alpha(0.)),
            RenderLayers::layer(MASK_LAYER),
        ))
        .insert(Name::new("mask_top"))
        .insert(MaskTop);
}

pub(crate) const GRADIENT_NORMALIZED_HEIGHT: f32 = 0.05;

pub(crate) const RING_OFFSET: f32 = 0.2;

pub(crate) const TOP_MASK_HEIGHT: f32 = 0.2;

fn update_mask(
    mut mask_bottom: Query<(&mut Fill, &mut Path), (With<MaskBottom>, Without<MaskTop>)>,
    mut mask_top: Query<(&mut Fill, &mut Path), (With<MaskTop>,Without<MaskBottom>)>,
    cams: Query<&Projection, With<GameCamera>>,
    chart: Res<GameChart>,
    time: Res<GameTime>,
    mut generated_mask: ResMut<GeneratedMaskTheme>,
) -> Result<()> {
    let Ok(game) = cams.single() else {
        return Ok(());
    };
    let Projection::Orthographic(game) = game else {
        return Err("GameCamera must use OrthographicProjection".into());
    };
    let area = game.area;
    let gradient_height = area.height() * GRADIENT_NORMALIZED_HEIGHT;

    if let Some(index) = chart.theme_control.pair(**time).0.map(|t| t.value) {
        if Some(index) == generated_mask.0 {
            return Ok(());
        } else {
            generated_mask.0 = Some(index);
        }
    }
    let Some(color) = chart
        .theme_at(**time)
        .ok()
        .map(|t| colorrgba_to_color(t.this.color.background))
    else {
        return Ok(());
    };

    // 更新 bottom mask
    if let Ok((mut fill, mut path)) = mask_bottom.get_single_mut() {
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
        let gradient_start = Vec2::new(0., -gradient_height);
        let gradient_end = Vec2::new(0., 0.);
        fill.brush = {
            let mut linear = LinearGradient::new_empty(gradient_start, gradient_end);
            linear.add_stop(0., color.with_alpha(1.));
            linear.add_stop(1., color.with_alpha(0.));
            let gradient: Gradient = linear.into();
            gradient.into()
        };
        *path = GeometryBuilder::new()
            .add(&Rectangle::new(mask_non_transparent_rect))
            .add(&Rectangle::new(gradient_rect))
            .build();
    }

    // 更新 top mask
    if let Ok((mut fill, mut path)) = mask_top.get_single_mut() {
        let mask_height = area.height() * TOP_MASK_HEIGHT;
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
        let gradient_start = Vec2::new(0., area.max.y - mask_height);
        let gradient_end = Vec2::new(0., area.max.y - mask_height + gradient_height);
        fill.brush = {
            let mut linear = LinearGradient::new_empty(gradient_start, gradient_end);
            linear.add_stop(0., color.with_alpha(0.));
            linear.add_stop(1., color.with_alpha(1.));
            let gradient: Gradient = linear.into();
            gradient.into()
        };
        *path = GeometryBuilder::new()
            .add(&Rectangle::new(mask_non_transparent_rect))
            .add(&Rectangle::new(gradient_rect))
            .build();
    }
    Ok(())
}
