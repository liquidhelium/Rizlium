use bevy::prelude::*;
use rizlium_render::LoadChartEvent;

use crate::{hotkeys::{HotkeysExt, HotkeyListener}, ActionsExt, PendingDialog, open_dialog, extensions::MenuExt, menu::{self}};
pub struct Game;

impl Plugin for Game {
    fn build(&self, app: &mut App) {
        use KeyCode::*;
        app.register_action("game.load_chart", load_chart)
            .register_action("game.open_dialog", open_dialog_and_load_chart)
            .register_hotkey(HotkeyListener::new_global("game.open_dialog".into(), [ControlLeft, O]))
            .menu_context(|mut ctx| {
                ctx.with_sub_menu("file", "File".into(), 0, |mut ctx| {
                    ctx.add("open_chart", "Open".into(), menu::Button::new("game.open_dialog".into()), 0);
                    // ctx.add("id", name, item, piority)
                });
            });
    }
}

fn load_chart(path: In<String>, mut load: EventWriter<LoadChartEvent>, _to_recent_file: () /* todo */) {
    load.send(LoadChartEvent(path.0));
}

fn open_dialog_and_load_chart(mut dialog: ResMut<PendingDialog>) {
    open_dialog(&mut dialog)
}