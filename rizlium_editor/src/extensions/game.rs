use bevy::prelude::*;
use rizlium_render::LoadChartEvent;

use crate::{
    extensions::MenuExt,
    hotkeys::{HotkeyListener, HotkeysExt},
    menu, open_dialog, ActionsExt, PendingDialog,
};
pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        use time_systems::*;
        use KeyCode::*;
        app.register_action("game.load_chart", load_chart)
            .register_action("game.open_dialog", open_dialog_and_load_chart)
            .register_action("game.time.advance", advance_time)
            .register_action("game.time.rewind", rewind_time)
            .register_action("game.time.toggle_pause", toggle_pause)
            .register_hotkey(HotkeyListener::new_global(
                "game.open_dialog",
                [ControlLeft, O],
            ))
            .register_hotkey(HotkeyListener::new_global(
                "game.time.advance",
                [Right],
            ))
            .register_hotkey(HotkeyListener::new_global(
                "game.time.rewind",
                [Left],
            ))
            .register_hotkey(HotkeyListener::new_global(
                "game.time.toggle_pause",
                [Space],
            ))
            .menu_context(|mut ctx| {
                ctx.with_sub_menu("file", "File".into(), 0, |mut ctx| {
                    ctx.add(
                        "open_chart",
                        "Open".into(),
                        menu::Button::new("game.open_dialog".into()),
                        0,
                    );
                    // ctx.add("id", name, item, piority)
                });
            });
    }
}

fn load_chart(
    path: In<String>,
    mut load: EventWriter<LoadChartEvent>,
    _to_recent_file: (), /* todo */
) {
    load.send(LoadChartEvent(path.0));
}

fn open_dialog_and_load_chart(mut dialog: ResMut<PendingDialog>) {
    open_dialog(&mut dialog)
}

mod time_systems {
    const SINGLE_TIME: f32 = 1.0;
    use rizlium_render::TimeControlEvent::*;

    use crate::EditorCommands;
    pub fn advance_time(mut commands: EditorCommands) {
        commands.time_control(Advance(SINGLE_TIME))
    }
    pub fn rewind_time(mut commands: EditorCommands) {
        commands.time_control(Advance(-SINGLE_TIME))
    }
    pub fn toggle_pause(mut commands: EditorCommands) {
        commands.time_control(Toggle)
    }
}
