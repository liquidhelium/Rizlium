use egui::{Color32, Context, Visuals};
use egui_dock::TabInteractionStyle;

pub fn top_bar_theme() -> Visuals {
    Visuals {
        widgets: egui::style::Widgets {
            inactive: egui::style::WidgetVisuals {
                weak_bg_fill: rgba(50, 50, 50, 0.0),
                bg_stroke: egui::Stroke::new(0.0, rgba(200, 200, 200, 0.0)),

                ..Visuals::dark().widgets.inactive
            },
            hovered: egui::style::WidgetVisuals {
                weak_bg_fill: rgba(100, 100, 100, 0.4),
                bg_stroke: egui::Stroke::new(1.0, rgba(200, 200, 200, 0.0)),

                ..Visuals::dark().widgets.hovered
            },
            ..Default::default()
        },
        ..Visuals::dark()
    }
}

pub fn tab_theme(ctx: &Context) -> egui_dock::Style {
    egui_dock::Style {
        separator: egui_dock::SeparatorStyle {
            width: 0.5,
            ..Default::default()
        },
        main_surface_border_rounding: 0.0.into(),
        tab_bar: egui_dock::TabBarStyle {
            corner_radius: 0.0.into(),
            ..egui_dock::Style::from_egui(&ctx.style()).tab_bar
        },
        tab: egui_dock::TabStyle {
            active: TabInteractionStyle {
                outline_color: rgba(0, 0, 0, 0.0),
                corner_radius: 0.0.into(),
                ..egui_dock::Style::from_egui(&ctx.style()).tab.active
            },
            focused: TabInteractionStyle {
                outline_color: rgba(0, 0, 0, 0.0),
                corner_radius: 0.0.into(),
                ..egui_dock::Style::from_egui(&ctx.style()).tab.focused
            },
            hovered: TabInteractionStyle {
                outline_color: rgba(0, 0, 0, 0.0),
                bg_fill: rgba(37, 37, 37, 1.0),
                corner_radius: 0.0.into(),
                ..egui_dock::Style::from_egui(&ctx.style()).tab.hovered
            },
            inactive: TabInteractionStyle {
                outline_color: rgba(0, 0, 0, 0.0),
                corner_radius: 0.0.into(),
                ..egui_dock::Style::from_egui(&ctx.style()).tab.inactive
            },
            ..egui_dock::Style::from_egui(&ctx.style()).tab
        },
        ..egui_dock::Style::from_egui(&ctx.style())
    }
}

fn rgba(arg_1: i32, arg_2: i32, arg_3: i32, arg_4: f64) -> egui::Color32 {
    Color32::from_rgba_unmultiplied(arg_1 as u8, arg_2 as u8, arg_3 as u8, (arg_4 * 255.0) as u8)
}
