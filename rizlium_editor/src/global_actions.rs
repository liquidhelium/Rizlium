use bevy::{input::mouse::MouseWheel, prelude::*};

use leafwing_input_manager::{
    prelude::InputMap,
    user_input::UserInput,
    Actionlike,
};

use crate::{hotkeys::HotkeyContext, EditorCommands};
#[derive(Actionlike, Reflect, Clone)]
pub enum GlobalEditorAction {
    OpenChartDialog,
    TogglePause,
    EnableScrollTime,
    AdvanceTime,
    RewindTime,
    SaveDocument,
}

impl GlobalEditorAction {
    pub fn default_map() -> InputMap<Self> {
        use GlobalEditorAction::*;
        use KeyCode::*;
        InputMap::new([
            (UserInput::from(Space), TogglePause),
            (ControlLeft.into(), EnableScrollTime),
            (Right.into(), AdvanceTime),
            (Left.into(), RewindTime),
        ])
    }
}

pub fn dispatch(
    context: HotkeyContext<GlobalEditorAction>,
    mut scroll_events: EventReader<MouseWheel>,
    mut commands: EditorCommands,
) {
    use rizlium_render::TimeControlEvent::*;
    use GlobalEditorAction::*;
    const SINGLE_TIME: f32 = 1.;
    const SCROLL_SPEED: f32 = 0.1;
    if context.pressed(EnableScrollTime) {
        for scroll in scroll_events.iter() {
            commands.time_control(Advance(SINGLE_TIME * SCROLL_SPEED * scroll.y));
        }
    } else if context.just_pressed(AdvanceTime) {
        commands.time_control(Advance(SINGLE_TIME));
    } else if context.just_pressed(RewindTime) {
        commands.time_control(Advance(-SINGLE_TIME));
    } else if context.just_pressed(OpenChartDialog) {
        commands.open_dialog_and_load_chart();
    } else if context.just_pressed(TogglePause) {
        commands.time_control(Toggle);
    }
}
