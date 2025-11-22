pub mod command_panel;
pub mod debug_flycam;
pub mod docking;
mod editing;
/// 半废弃
mod explorer;
mod game;
pub mod i18n;
mod inspector;
/// 半废弃
mod project_guide;
use self::{
    command_panel::CommandPanel, docking::Docking, editing::Editing, game::Game, i18n::I18nPlugin,
    inspector::Inspector, project_guide::ProjectGuideExtension,
};
use crate::extensions::{debug_flycam::DebugCamExtension, hierarchy_inspector::HierarchyInspector};
use bevy::prelude::{App, Plugin};
pub struct ExtensionsPlugin;
impl Plugin for ExtensionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            I18nPlugin,
            Game,
            Docking,
            CommandPanel,
            Editing,
            Inspector,
            DebugCamExtension,
            HierarchyInspector,
        ));
    }
}
/// Abstract hierarchy for current editing chart
mod hierarchy_inspector {
    use bevy::prelude::*;
    use helium_framework::prelude::TabRegistrationExt as _;
    use rizlium_chart::editing::chart_path::CanvasPath;
    use rizlium_render::ChartProvider as _;
    use rust_i18n::t;

    use crate::extensions::inspector::SelectedItem;

    pub struct HierarchyInspector;
    impl bevy::prelude::Plugin for HierarchyInspector {
        fn build(&self, app: &mut bevy::prelude::App) {
            app.register_tab(
                "hierarchy",
                t!("hierarchy.tab"),
                hierarchy_ui,
                crate::project::ProjectState::has_chart_system(),
            );
        }
    }
    fn hierarchy_ui(
        InMut(mut ui): InMut<egui::Ui>,
        chart: Res<crate::project::ProjectState>,
        mut select: ResMut<SelectedItem>,
    ) {
        // chart must exist.
        let chart = chart.chart();
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show_viewport(ui, |ui, _rect| {
                ui.collapsing("Canvases", |ui| {
                    for (i, canvas) in chart.canvases.iter().enumerate() {
                        if ui
                            .selectable_label(
                                matches!(
                                    select.item,
                                    Some(crate::extensions::inspector::ChartItem::Canvas(ref p))
                                        if p.0 == i
                                ),
                                format!("Canvas {}", i),
                            )
                            .clicked()
                        {
                            // select this canvas
                            select.item = Some(crate::extensions::inspector::ChartItem::Canvas(
                                CanvasPath(i),
                            ));
                        }
                    }
                });
                ui.collapsing("Lines", |ui| {
                    for (i, line) in chart.lines.iter().enumerate() {
                        if ui
                            .selectable_label(
                                matches!(
                                    select.item,
                                    Some(crate::extensions::inspector::ChartItem::Line(ref p))
                                        if p.0 == i
                                ),
                                format!("Line {}", i),
                            )
                            .clicked()
                        {
                            // select this line
                            select.item = Some(crate::extensions::inspector::ChartItem::Line(
                                rizlium_chart::editing::chart_path::LinePath(i),
                            ));
                        }
                    }
                });
            });
    }
}
