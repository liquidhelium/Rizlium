use bevy::{ecs::system::SystemParam, prelude::*, winit::WinitWindows};
#[derive(Event)]
pub struct DragWindowRequested;

#[derive(Event)]
pub struct MaximizeRequested;

#[derive(Event)]
pub struct MinimizeRequested;

#[derive(SystemParam)]
struct ExtraWindowEvents<'w, 's> {
    drag_window: EventReader<'w, 's, DragWindowRequested>,
    maximize: EventReader<'w, 's, MaximizeRequested>,
    minimize: EventReader<'w, 's, MinimizeRequested>,
}

macro_rules! fn_events {
    ($($name:ident),*) => {
        fn all_empty(&self) -> bool {
            $(self.$name.is_empty())&&*
        }
        $(fn $name(&mut self) -> bool {
            if !self.$name.is_empty() {
                self.$name.clear();
                return true;
            }
            false
        })*
    };
}
impl ExtraWindowEvents<'_,'_> {
    fn_events!(drag_window, maximize, minimize);
}

fn dispatch_events(
    mut windows: Query<Entity, With<Window>>,
    winit_windows: NonSendMut<WinitWindows>,
    mut events: ExtraWindowEvents,
) {
    if events.all_empty() {
        return;
    }
    
    let entity = windows.single_mut();
    let winit_windows = winit_windows
        .get_window(entity)
        .expect("invalid window entity used");
    if events.drag_window() {
        info!("dragging window");
        let _ = winit_windows.drag_window().map_err(|err| warn!("{}", err));
    }
    if events.maximize() {
        winit_windows.set_maximized(true);
    }
    if events.minimize() {
        winit_windows.set_minimized(true);
    }
}

pub struct ExtraWindowControlPlugin;

impl Plugin for ExtraWindowControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DragWindowRequested>()
            .add_event::<MaximizeRequested>()
            .add_event::<MinimizeRequested>();
        app.add_systems(Update, dispatch_events);
    }
}
