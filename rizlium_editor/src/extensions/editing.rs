// 引入自定义的 `helium_framework` 框架的核心预设。
use helium_framework::prelude::*;

// 引入当前 crate 中的其他模块。
use crate::{project::ProjectState, MainMenuContext};

// 从当前模块的子模块中引入 specific 函数。
use self::{note::note_editor_vertical, tool_config_window::tool_config};
// 引入 Bevy 引擎的核心预设模块。
use bevy::prelude::*;
// 引入 egui 库的相关组件，用于 UI 绘制和交互。
use egui::{emath::RectTransform, vec2, Color32, Sense, Stroke, Ui, UiBuilder};
// 引入 `rizlium_chart` crate 中的谱面和编辑历史相关定义。
use rizlium_chart::{chart::Spline, editing::EditHistory};
// 引入渲染模块中的 `ChartProvider` trait 和 `GameTime` 资源。
use rizlium_render::{ChartProvider, GameTime};
// 引入国际化宏 `t!`。
use rust_i18n::t;
// 引入样条曲线视图组件。
use spline::SplineView;

// 声明并公开 `note` 子模块。
pub mod note;
// 声明私有的 `spline` 子模块。
mod spline;
// 声明并公开 `timeline` 子模块。
pub mod timeline;
// 声明私有的 `tool_config_window` 子模块。
mod tool_config_window;
// 声明私有的 `tool_select_bar` 子模块。
mod tool_select_bar;
// 声明私有的 `undo_redo` 子模块。
mod undo_redo;
// 声明并公开 `world_view` 子模块。
pub mod world_view;

/// `Editing` 是一个 Bevy 插件，用于聚合所有与编辑相关的功能。
pub struct Editing;

impl Plugin for Editing {
    fn build(&self, app: &mut App) {
        // --- 注册编辑相关的标签页 ---
        // 每个标签页都使用 `ProjectState::has_chart_system()` 作为运行条件，
        // 这意味着只有在成功加载谱面后，这些标签页才会被创建和显示。
        app.register_tab(
            "edit.note", // 标签页的唯一 ID。
            t!("edit.note.tab"), // 标签页的显示名称（从翻译文件中获取）。
            note_window, // 渲染该标签页 UI 的系统。
            ProjectState::has_chart_system(), // 运行条件。
        )
        .register_tab(
            "edit.spline",
            t!("edit.spline.tab"),
            spline_edit,
            ProjectState::has_chart_system(),
        )
        .register_tab(
            "edit.tool_config",
            t!("edit.tool_config.tab"),
            tool_config,
            ProjectState::has_chart_system(),
        );

        // 添加 `WorldViewPlugin`，它负责 3D 谱面视图的渲染和交互。
        app.add_plugins(world_view::WorldViewPlugin)
            // 初始化 `ChartEditHistory` 资源，用于存储撤销/重做操作。
            .init_resource::<ChartEditHistory>();

        // --- 注册撤销/重做动作和菜单项 ---
        // 注册 `undo` 和 `redo` 系统为反射系统，以便可以通过 ID 调用。
        app.reflect_system("edit.undo", t!("edit.undo.desc"), undo_redo::undo);
        app.reflect_system("edit.redo", t!("edit.redo.desc"), undo_redo::redo);
        // 使用 `use` 语句简化 `KeyCode` 的使用。
        use KeyCode::*;
        // 注册全局快捷键 `Ctrl+Z` 用于撤销，`Ctrl+Y` 用于重做。
        app.register_hotkey("edit.undo", [Hotkey::new_global([ControlLeft, KeyZ])])
            .register_hotkey("edit.redo", [Hotkey::new_global([ControlLeft, KeyY])])
            // 在主菜单中创建一个名为 "Edit" 的子菜单。
            .register_submenu::<MainMenuContext>("edit", t!("edit.name"))
            // 在 "Edit" 子菜单下添加 "Undo" 和 "Redo" 命令。
            .register_command::<MainMenuContext>("edit/undo", t!("edit.undo.name"), "edit.undo")
            .register_command::<MainMenuContext>("edit/redo", t!("edit.redo.name"), "edit.redo");
    }
}

/// `ChartEditHistory` 是一个 Bevy 资源，它包装了 `rizlium_chart` 中的 `EditHistory`，
/// 用于在整个应用中管理谱面编辑的撤销/重做栈。
#[derive(Deref, DerefMut, Resource, Default)]
pub struct ChartEditHistory(EditHistory);

/// `note_window` 是一个 Bevy `widget` 系统，用于渲染 Note 编辑器窗口。
fn note_window(
    InMut(ui): InMut<Ui>,
    chart: Res<ProjectState>,
    mut focused: Local<usize>, // 使用 `Local` 变量来在系统调用之间保持状态（当前聚焦的谱面线索引）。
    mut scale: Local<f32>, // 保持缩放比例。
    mut row_width: Local<f32>, // 保持行宽。
    time: Res<GameTime>,
) {
    // 初始化 `Local` 变量的默认值。
    if *scale == 0. {
        *scale = 200.;
    }
    if *row_width == 0. {
        *row_width = 50.
    }
    // 在 UI 顶部添加一些控制滑块。
    ui.scope(|ui| {
        ui.style_mut().spacing.slider_width = 500.;

        // 用于选择当前编辑的谱面线的滑块。
        ui.add(egui::Slider::new(
            &mut *focused,
            0..=(chart.chart().lines.len() - 1),
        ));
        // 用于控制时间轴缩放的对数滑块。
        ui.add(egui::Slider::new(&mut *scale, 1.0..=2000.0).logarithmic(true));
        // 用于控制音符行宽的滑块。
        ui.add(egui::Slider::new(&mut *row_width, 10.0..=200.0));
    });
    // 调用实际的 Note 编辑器 UI 渲染函数。
    note_editor_vertical(
        ui,
        Some(0), // TODO: 这个参数的用途是什么？
        chart.chart()
            .lines
            .iter()
            .map(|l| l.notes.as_slice())
            .enumerate()
            .collect::<Vec<_>>()
            .as_slice(),
        **time,
        &mut scale,
        *row_width,
        200.,
    )
}

/// `spline_edit` 是一个 Bevy `widget` 系统，用于渲染样条曲线（变速线）编辑器。
pub fn spline_edit(
    InMut(ui): InMut<Ui>,
    chart: Res<ProjectState>,
    mut current: Local<usize>, // 当前编辑的变速线索引。
    mut visible_rect: Local<Option<egui::Rect>>, // 可视区域，用于平移和缩放。
    _external: Local<Spline<f32>>, // TODO: 这个参数的用途是什么？
) {
    let mut show_first = false;
    // 添加一个滑块用于选择要编辑的变速线（canvas）。
    ui.scope(|ui| {
        ui.style_mut().spacing.slider_width = 500.;

        show_first |= ui
            .add(egui::Slider::new(
                &mut *current,
                0..=(chart.chart().canvases.len() - 1),
            ))
            .changed();
    });
    // `inner` 包含了 `spline_view.ui` 的响应和 `spline_view` 本身。
    let (res, spline_view) = {
        let max_rect = ui.available_rect_before_wrap();
        ui.allocate_ui_at_rect(max_rect, |ui| {
            let spline = &chart.chart().canvases[*current].speed;
            // 创建 `SplineView` 实例。
            let spline_view =
                SplineView::new(ui, spline, *visible_rect, spline::Orientation::Horizontal);
            // 渲染样条曲线 UI 并获取响应。
            let response = spline_view.ui(ui);
            let spline_area = spline_view.spline_area();
            
            // --- 绘制小地图（Minimap） ---
            const WIDTH: f32 = 80.0;
            const RATIO: f32 = 9. / 16.;
            let indicating_rect_full = egui::Rect::from_min_size(
                response.rect.min + vec2(20., 20.),
                vec2(WIDTH, WIDTH * RATIO),
            );
            // 创建一个从完整样条曲线区域到小地图区域的坐标变换。
            let spline_to_interact = RectTransform::from_to(spline_area, indicating_rect_full);
            // 使用变换来计算当前可视区域在小地图上的位置。
            let indicating_rect_inner =
                spline_to_interact.transform_rect(spline_view.visible_spline_area());
            // 绘制小地图的边框。
            ui.painter_at(response.rect).rect(
                indicating_rect_full,
                0.,
                Color32::from_white_alpha(20),
                Stroke::new(1., Color32::BLACK),
                egui::StrokeKind::Middle,
            );
            let mut alpha = 20;
            // 使小地图上的可视区域矩形可拖动。
            let inner_interact = ui.interact(
                indicating_rect_inner,
                ui.id().with("indicating_rect_inner"),
                Sense::drag(),
            );
            if inner_interact.hovered() {
                alpha += 10;
            }
            // 绘制小地图上的可视区域矩形。
            ui.painter_at(response.rect).rect_filled(
                indicating_rect_inner,
                0.,
                Color32::from_white_alpha(alpha),
            );
            // 如果用户拖动了小地图上的可视区域矩形，则更新主视图的 `visible_rect`。
            if inner_interact.dragged() {
                let transformed = spline_to_interact
                    .inverse()
                    .transform_rect(indicating_rect_inner.translate(inner_interact.drag_delta()));
                *visible_rect = Some(transformed);
            }

            (response, spline_view)
        })
    }
    .inner;

    // --- 处理主视图的平移 ---
    // 如果用户拖动了主样条曲线视图，则平移 `visible_rect`。
    if res.dragged() {
        let scale = spline_view.view2visible().scale();
        let delta = (-res.drag_delta()) * scale;
        let rect = visible_rect.unwrap_or(spline_view.visible_spline_area());
        *visible_rect = Some(rect.translate(delta));
    }

    // 如果用户切换了正在编辑的样条曲线，则重置视图。
    if show_first {
        *visible_rect = None;
    }
}
