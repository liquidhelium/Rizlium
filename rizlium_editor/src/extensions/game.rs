use bevy::prelude::*;
use rizlium_render::LoadChartEvent;

use crate::{
    extensions::MenuExt,
    hotkeys::{Hotkey, HotkeysExt},
    menu, open_dialog, ActionsExt, PendingDialog,
};
pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        use time_systems::*;
        use KeyCode::*;
        app.register_action("game.load_chart", "Load chart file",load_chart)
            .register_action("game.open_dialog", "Open a dialog to pick chart file and load it", open_dialog_and_load_chart)
            .register_action("game.time.advance", "Advance game time",advance_time)
            .register_action("game.time.rewind", "Rewind game time", rewind_time)
            .register_action("game.time.toggle_pause", "Pause or resume game",toggle_pause)
            .register_hotkey(
                "game.open_dialog",
                Hotkey::new_global([ControlLeft, O]),
            )
            .register_hotkey("game.time.advance", Hotkey::new_global([Right]))
            .register_hotkey("game.time.rewind", Hotkey::new_global([Left]))
            .register_hotkey(
                "game.time.toggle_pause",
                Hotkey::new_global([Space]),
            )
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
