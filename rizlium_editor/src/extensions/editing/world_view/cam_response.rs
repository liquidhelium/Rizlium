use bevy::{prelude::*, render::view::RenderLayers, math::Ray3d};
use bevy_mod_raycast::immediate::{Raycast, RaycastSettings};
use rizlium_render::ChartLine;
use strum::EnumIs;

use super::WorldCam;

pub struct RaycastPlugin;

impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenMouseEvent>()
            .add_event::<WorldMouseEvent>()
            .add_systems(
                PreUpdate,
                ray_cast.run_if(resource_exists_and_changed::<Events<ScreenMouseEvent>>),
            )
            .add_systems(PostUpdate, add_pick_to_lines);
    }
}

#[derive(Event, Debug)]
pub struct ScreenMouseEvent(pub MouseEvent);

#[derive(Event, Debug)]
pub struct WorldMouseEvent {
    pub event: MouseEvent,
    pub casted_entity: Option<Entity>,
}

#[derive(Clone, Debug)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub button: Option<MouseButton>,
    pub pos: Vec3,
}

#[derive(Clone, Debug, EnumIs)]
pub enum MouseEventType {
    Hover,
    Click(ClickEventType),
    Drag(DragEventType),
}

#[derive(Clone, Debug)]
pub enum DragEventType {
    DragStarted,
    DragEnded,
    Dragging(Vec2),
}

#[derive(Clone, Debug)]
pub enum ClickEventType {
    Single,
    Double,
    Triple,
}

#[derive(Component)]
pub struct CamResponse(Option<MouseEvent>);

fn to_ray(pixel: Vec2, cam: &Camera, trans: &GlobalTransform) -> Option<Ray3d> {
    cam.viewport_to_world(trans, pixel).map(Ray3d::from)
}

fn ray_cast(
    mut raycast: Raycast,
    camera: Query<(&Camera, &GlobalTransform, Option<&RenderLayers>), With<WorldCam>>,
    mut meshes: Query<(Entity, Option<&RenderLayers>, &mut CamResponse)>,
    mut screen_mouse_events: EventReader<ScreenMouseEvent>,
    mut world_mouse_events: EventWriter<WorldMouseEvent>,
) {
    let (cam, trans, cam_layers) = camera.single();
    screen_mouse_events.read().for_each(|ev| {
        let mut owned_event;
        let [(cast_entity, cast_data)] = raycast.cast_ray(
            {
                let Some(ray) = to_ray(ev.0.pos.xy(), cam, trans) else {
                    return;
                };
                owned_event = ev.0.clone();
                owned_event.pos = ray.origin;
                ray
            },
            &RaycastSettings::default()
                .always_early_exit()
                .with_filter(&|entity| {
                    meshes.get(entity).is_ok_and(|(_, layers, _)| {
                        let default_layer = RenderLayers::layer(0);
                        let layers = layers.unwrap_or(&default_layer);
                        let cam_layers = cam_layers.unwrap_or(&default_layer);
                        layers.intersects(cam_layers)
                    })
                }),
        ) else {
            world_mouse_events.send(WorldMouseEvent {
                event: owned_event.clone(),
                casted_entity: None,
            });
            return;
        };
        world_mouse_events.send(WorldMouseEvent {
            event: owned_event.clone(),
            casted_entity: Some(cast_entity.clone()),
        });
        owned_event.pos = cast_data.position();
        meshes.par_iter_mut().for_each(|(entity, _, mut response)| {
            if entity == *cast_entity {
                response.0 = Some(owned_event.clone());
            } else if response.0.is_some() {
                response.0 = None;
            }
        });
    })
}

fn add_pick_to_lines(mut commands: Commands, lines: Query<Entity, Added<ChartLine>>) {
    lines.iter().for_each(|entity| {
        commands.entity(entity).insert(CamResponse(None));
    });
}
