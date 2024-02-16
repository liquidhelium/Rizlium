use bevy::prelude::*;
use bevy_mod_raycast::{
    immediate::{Raycast, RaycastSettings},
    primitives::Ray3d,
};

use super::WorldCam;

pub struct RaycastPlugin;

impl Plugin for RaycastPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ScreenMouseEvent>()
            .add_systems(PreUpdate, ray_cast.run_if(resource_exists_and_changed::<Events<ScreenMouseEvent>>()));
    }
}

#[derive(Event, Debug)]
pub struct ScreenMouseEvent(pub MouseEvent);

#[derive(Clone, Debug)]
pub struct MouseEvent {
    pub event_type: MouseEventType,
    pub button: Option<MouseButton>,
    pub pos: Vec3,
}

#[derive(Clone, Debug)]
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
    camera: Query<(&Camera, &GlobalTransform), With<WorldCam>>,
    mut meshes: Query<(Entity, &mut CamResponse)>,
    mut screen_mouse_events: EventReader<ScreenMouseEvent>,
) {
    let (cam, trans) = camera.single();
    screen_mouse_events.read().for_each(|ev| {
        let [(cast_entity, cast_data)] = raycast.cast_ray(
            {
                let Some(ray) = to_ray(ev.0.pos.xy(), cam, trans) else {
                    return;
                };
                ray
            },
            &RaycastSettings::default().always_early_exit(),
        ) else {
            return;
        };
        let mut owned_event = ev.0.clone();
        owned_event.pos = cast_data.position();
        meshes.par_iter_mut().for_each(|(entity,mut response)| {
            if entity == *cast_entity {
                response.0 = Some(owned_event.clone());
            }
            else if response.0.is_some() {
                response.0 = None;
            }
        });
    })
}
