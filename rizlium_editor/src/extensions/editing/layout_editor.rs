//! 谱面配置编辑器
//!
//! 用于编辑谱面中的音符布局（LayoutNote）的竖直时间线编辑器。
//! 特点：
//! - 竖直方向的时间线
//! - 支持吸附到网格
//! - 支持点选和框选
//! - 支持拖拽移动音符
//! - 支持undo/redo

use std::ops::RangeInclusive;

use bevy::prelude::*;
use egui::{Align2, Color32, FontId, Id, Rect, Sense, Stroke, Ui, pos2, vec2};
use helium_framework::prelude::*;
use rizlium_chart::prelude::{LayoutNote, NoteKind};
use rizlium_render::ChartProvider as _;

use crate::extensions::editing::ChartEditHistory;
use crate::project::ProjectState;
use crate::time_and_audio::TimeControlEvent;

use super::world_view::snapping::{SnappingConfig, SnappingContext};

pub struct LayoutEditorPlugin;

impl Plugin for LayoutEditorPlugin {
    fn build(&self, app: &mut App) {
        app.register_tab(
            "edit.layout",
            t!("edit-layout-tab"),
            layout_editor_tab,
            ProjectState::has_chart_system(),
        )
        .init_resource::<LayoutEditorState>();
    }
}

/// 当前选择的工具
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LayoutTool {
    #[default]
    Pencil,
    Select,
}

/// 当前选择的音符类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectedNoteKind {
    #[default]
    Tap,
    Drag,
    Hold,
}

impl SelectedNoteKind {
    pub fn to_note_kind(self) -> NoteKind {
        match self {
            SelectedNoteKind::Tap => NoteKind::Tap,
            SelectedNoteKind::Drag => NoteKind::Drag,
            SelectedNoteKind::Hold => NoteKind::Hold { end: 1.0 },
        }
    }
}

/// 编辑器状态资源
#[derive(Resource, Default)]
pub struct LayoutEditorState {
    /// 当前工具
    pub tool: LayoutTool,
    /// 当前选择的音符类型（用于创建新音符）
    pub note_kind: SelectedNoteKind,
    /// 选中的音符索引列表
    pub selected_notes: Vec<usize>,
    /// 可见时间范围
    pub time_range: Option<RangeInclusive<f32>>,
    /// 可见x范围
    pub x_range: Option<RangeInclusive<f32>>,
    /// 跟随游标
    pub follow_cursor: bool,
    /// 拖拽状态
    drag_state: Option<DragState>,
    /// 框选状态
    box_select_state: Option<BoxSelectState>,
}

#[derive(Clone)]
struct DragState {
    /// 拖拽开始时选中音符的原始位置
    original_positions: Vec<(f32, f32)>, // (time, x)
    /// 拖拽起始点（屏幕坐标）
    start_pos: egui::Pos2,
}

#[derive(Clone)]
struct BoxSelectState {
    start_pos: egui::Pos2,
    current_pos: egui::Pos2,
}

/// x 的有效范围
const X_MIN: f32 = -450.0;
const X_MAX: f32 = 450.0;
/// 时间有效范围
const TIME_MIN: f32 = 0.0;
const TIME_MAX: f32 = 300.0; // 可以调整

/// 音符显示的半径
const NOTE_RADIUS: f32 = 12.0;

fn layout_editor_tab(
    InMut(ui): InMut<Ui>,
    mut chart_state: ResMut<ProjectState>,
    mut state: ResMut<LayoutEditorState>,
    game_time: Res<rizlium_render::GameTime>,
    mut time_control: MessageWriter<TimeControlEvent>,
    snapping_config: Res<SnappingConfig>,
    mut chart_edit_history: ResMut<ChartEditHistory>,
    mut toast: ResMut<ToastsStorage>,
    _actions: Actions,
) {
    let chart = chart_state.chart();
    let cursor_time = **game_time;

    // 初始化范围
    let time_range = state.time_range.clone().unwrap_or(0.0..=10.0);
    let x_range = state.x_range.clone().unwrap_or(X_MIN..=X_MAX);

    // 顶部工具栏
    let _toolbar_response = ui.horizontal(|ui| {
        ui.label(t!("edit-layout-tool"));
        if ui
            .selectable_label(state.tool == LayoutTool::Pencil, t!("edit-layout-pencil"))
            .clicked()
        {
            state.tool = LayoutTool::Pencil;
        }
        if ui
            .selectable_label(state.tool == LayoutTool::Select, t!("edit-layout-select"))
            .clicked()
        {
            state.tool = LayoutTool::Select;
        }
        ui.separator();
        ui.label(t!("edit-layout-note-kind"));
        if ui
            .selectable_label(state.note_kind == SelectedNoteKind::Tap, "Tap")
            .clicked()
        {
            state.note_kind = SelectedNoteKind::Tap;
        }
        if ui
            .selectable_label(state.note_kind == SelectedNoteKind::Drag, "Drag")
            .clicked()
        {
            state.note_kind = SelectedNoteKind::Drag;
        }
        if ui
            .selectable_label(state.note_kind == SelectedNoteKind::Hold, "Hold")
            .clicked()
        {
            state.note_kind = SelectedNoteKind::Hold;
        }
        ui.separator();
        if ui
            .selectable_label(state.follow_cursor, t!("edit-layout-follow"))
            .clicked()
        {
            state.follow_cursor = !state.follow_cursor;
        }
    });

    // 主编辑区域
    let available_rect = ui.available_rect_before_wrap();

    // 左侧时间轴区域
    let timeline_width = 60.0;
    let scrollbar_height = 30.0;

    let timeline_rect = Rect::from_min_size(
        available_rect.min,
        vec2(timeline_width, available_rect.height() - scrollbar_height),
    );

    // 主内容区域
    let content_rect = Rect::from_min_size(
        pos2(available_rect.min.x + timeline_width, available_rect.min.y),
        vec2(
            available_rect.width() - timeline_width,
            available_rect.height() - scrollbar_height,
        ),
    );

    // 底部滚动条区域
    let scrollbar_rect = Rect::from_min_size(
        pos2(
            available_rect.min.x,
            available_rect.max.y - scrollbar_height,
        ),
        vec2(available_rect.width(), scrollbar_height),
    );

    // 绘制背景
    ui.painter()
        .rect_filled(timeline_rect, 0.0, Color32::from_rgb(31, 31, 31));
    ui.painter()
        .rect_filled(content_rect, 0.0, Color32::from_rgb(40, 40, 40));
    ui.painter()
        .rect_filled(scrollbar_rect, 0.0, Color32::from_rgb(31, 31, 31));

    // 坐标转换函数
    let time_to_screen_y =
        |time: f32| -> f32 { egui::remap(time, time_range.clone(), content_rect.y_range()) };
    let screen_y_to_time =
        |y: f32| -> f32 { egui::remap(y, content_rect.y_range(), time_range.clone()) };
    let x_to_screen_x = |x: f32| -> f32 { egui::remap(x, x_range.clone(), content_rect.x_range()) };
    let screen_x_to_x =
        |sx: f32| -> f32 { egui::remap(sx, content_rect.x_range(), x_range.clone()) };

    // 绘制网格
    {
        let _snapping_context = SnappingContext::new(&snapping_config, Some(chart));

        // 水平网格线（时间）
        let time_step = 1.0 / snapping_config.time_divisor as f32;
        let start_time = (time_range.start() / time_step).floor() * time_step;
        let mut t = start_time;
        while t <= *time_range.end() {
            let y = time_to_screen_y(t);
            let alpha = if (t * snapping_config.time_divisor as f32).round() as i32 % 4 == 0 {
                100
            } else {
                40
            };
            ui.painter().hline(
                content_rect.x_range(),
                y,
                Stroke::new(1.0, Color32::from_white_alpha(alpha)),
            );
            t += time_step;
        }

        // 垂直网格线（x）
        let x_step = snapping_config.value_step;
        let start_x = (x_range.start() / x_step).floor() * x_step;
        let mut x = start_x;
        while x <= *x_range.end() {
            let sx = x_to_screen_x(x);
            let alpha = if (x / x_step).round() as i32 % 4 == 0 {
                100
            } else {
                40
            };
            ui.painter().vline(
                sx,
                content_rect.y_range(),
                Stroke::new(1.0, Color32::from_white_alpha(alpha)),
            );
            x += x_step;
        }
    }

    // 绘制游标（水平线）
    let cursor_y = time_to_screen_y(cursor_time);
    ui.painter().hline(
        content_rect.x_range(),
        cursor_y,
        Stroke::new(2.0, Color32::from_rgb(100, 150, 255)),
    );

    // 绘制时间轴刻度
    {
        let time_step = 1.0;
        let start_time = time_range.start().floor();
        let mut t = start_time;
        while t <= *time_range.end() {
            let y = time_to_screen_y(t);
            ui.painter().text(
                pos2(timeline_rect.right() - 5.0, y),
                Align2::RIGHT_CENTER,
                format!("{:.0}", t),
                FontId::proportional(12.0),
                Color32::WHITE,
            );
            ui.painter().hline(
                timeline_rect.x_range(),
                y,
                Stroke::new(1.0, Color32::from_white_alpha(80)),
            );
            t += time_step;
        }
    }

    // 绘制音符
    let layout_notes = &chart.layout_notes;
    for (idx, note) in layout_notes.iter().enumerate() {
        let screen_x = x_to_screen_x(note.x);
        let screen_y = time_to_screen_y(note.time);
        let pos = pos2(screen_x, screen_y);

        // 检查是否在可见区域内
        if !content_rect.contains(pos) {
            continue;
        }

        let is_selected = state.selected_notes.contains(&idx);
        let color = match note.kind {
            NoteKind::Tap => Color32::from_rgb(100, 200, 255),
            NoteKind::Drag => Color32::from_rgb(255, 200, 100),
            NoteKind::Hold { .. } => Color32::from_rgb(100, 255, 150),
        };
        let stroke_color = if is_selected {
            Color32::WHITE
        } else {
            Color32::from_white_alpha(100)
        };

        ui.painter()
            .circle(pos, NOTE_RADIUS, color, Stroke::new(2.0, stroke_color));
    }

    // 交互处理
    let content_response = ui.interact(
        content_rect,
        Id::new("layout_content"),
        Sense::click_and_drag(),
    );

    // 使用 None 作为 chart，因为我们的吸附功能不需要 chart 引用
    let snapping_context = SnappingContext::new(&snapping_config, None);

    // 鼠标位置转换为谱面坐标
    let mouse_pos = content_response.hover_pos();
    let (mouse_time, mouse_x) = if let Some(pos) = mouse_pos {
        let raw_time = screen_y_to_time(pos.y);
        let raw_x = screen_x_to_x(pos.x);
        let snapped_time = snapping_context.snap_time(raw_time);
        let snapped_x = snapping_context.snap_value(raw_x);
        (snapped_time, snapped_x)
    } else {
        (0.0, 0.0)
    };

    // 检测鼠标是否悬停在某个音符上
    let hovered_note_idx = mouse_pos.and_then(|pos| {
        layout_notes
            .iter()
            .enumerate()
            .find(|(_, note)| {
                let note_pos = pos2(x_to_screen_x(note.x), time_to_screen_y(note.time));
                note_pos.distance(pos) <= NOTE_RADIUS + 5.0
            })
            .map(|(idx, _)| idx)
    });

    // 绘制预览音符（铅笔工具）
    if state.tool == LayoutTool::Pencil && mouse_pos.is_some() && hovered_note_idx.is_none() {
        let preview_pos = pos2(x_to_screen_x(mouse_x), time_to_screen_y(mouse_time));
        if content_rect.contains(preview_pos) {
            let preview_color = match state.note_kind {
                SelectedNoteKind::Tap => Color32::from_rgba_unmultiplied(100, 200, 255, 100),
                SelectedNoteKind::Drag => Color32::from_rgba_unmultiplied(255, 200, 100, 100),
                SelectedNoteKind::Hold => Color32::from_rgba_unmultiplied(100, 255, 150, 100),
            };
            ui.painter().circle(
                preview_pos,
                NOTE_RADIUS,
                preview_color,
                Stroke::new(1.0, Color32::from_white_alpha(50)),
            );
        }
    }

    // 绘制框选矩形
    if let Some(box_state) = &state.box_select_state {
        let rect = Rect::from_two_pos(box_state.start_pos, box_state.current_pos);
        ui.painter().rect(
            rect,
            0.0,
            Color32::from_rgba_unmultiplied(100, 150, 255, 50),
            Stroke::new(1.0, Color32::from_rgb(100, 150, 255)),
            egui::StrokeKind::Middle,
        );
    }

    // 处理点击和拖拽
    let chart_mut = chart_state.chart_mut();

    if content_response.clicked_by(egui::PointerButton::Primary) {
        match state.tool {
            LayoutTool::Pencil => {
                // 如果没有点击到现有音符，创建新音符
                if hovered_note_idx.is_none() {
                    let clamped_time = mouse_time.clamp(TIME_MIN, TIME_MAX);
                    let clamped_x = mouse_x.clamp(X_MIN, X_MAX);
                    let new_note =
                        LayoutNote::new(clamped_time, clamped_x, state.note_kind.to_note_kind());

                    let command = rizlium_chart::editing::commands::InsertLayoutNote {
                        note: new_note,
                        at: None,
                    };
                    if let Err(err) = chart_edit_history.push(command, chart_mut) {
                        toast.error(err.to_string());
                    }
                } else if let Some(idx) = hovered_note_idx {
                    // 点击到现有音符，选中它
                    if ui.input(|i| i.modifiers.ctrl) {
                        // Ctrl+点击：切换选中
                        if state.selected_notes.contains(&idx) {
                            state.selected_notes.retain(|&i| i != idx);
                        } else {
                            state.selected_notes.push(idx);
                        }
                    } else {
                        state.selected_notes = vec![idx];
                    }
                }
            }
            LayoutTool::Select => {
                if let Some(idx) = hovered_note_idx {
                    if ui.input(|i| i.modifiers.ctrl) {
                        if state.selected_notes.contains(&idx) {
                            state.selected_notes.retain(|&i| i != idx);
                        } else {
                            state.selected_notes.push(idx);
                        }
                    } else if !state.selected_notes.contains(&idx) {
                        state.selected_notes = vec![idx];
                    }
                } else {
                    // 点击空白处清除选择
                    state.selected_notes.clear();
                }
            }
        }
    } else if content_response.clicked_by(egui::PointerButton::Secondary) || content_response.dragged_by(egui::PointerButton::Secondary) {
        if let Some(idx) = hovered_note_idx {
            if let Err(err) = chart_edit_history.push(
                rizlium_chart::editing::commands::RemoveLayoutNote {
                    note_path: rizlium_chart::editing::LayoutNotePath::new(idx),
                },
                chart_mut,
            ) {
                toast.error(err.to_string());
            }
        } else if !content_response.dragged_by(egui::PointerButton::Secondary) {
            // 右键空白处清除选择
            state.selected_notes.clear();
        }
    }

    // 处理拖拽开始
    if content_response.drag_started_by(egui::PointerButton::Primary) {
        if state.tool == LayoutTool::Select && hovered_note_idx.is_none() {
            // 开始框选
            if let Some(pos) = mouse_pos {
                state.box_select_state = Some(BoxSelectState {
                    start_pos: pos,
                    current_pos: pos,
                });
            }
        } else if let Some(idx) = hovered_note_idx {
            // 开始拖拽音符
            if !state.selected_notes.contains(&idx) {
                state.selected_notes = vec![idx];
            }
            let original_positions: Vec<_> = state
                .selected_notes
                .iter()
                .filter_map(|&i| chart_mut.layout_notes.get(i).map(|n| (n.time, n.x)))
                .collect();
            state.drag_state = Some(DragState {
                original_positions,
                start_pos: mouse_pos.unwrap_or_default(),
            });
        }
    }

    // 处理拖拽中
    if content_response.dragged_by(egui::PointerButton::Primary) {
        if let Some(box_state) = &mut state.box_select_state {
            if let Some(pos) = mouse_pos {
                box_state.current_pos = pos;
            }
        } else if let Some(drag_state) = &state.drag_state {
            if let Some(current_pos) = mouse_pos {
                let delta_screen = current_pos - drag_state.start_pos;
                let delta_time = screen_y_to_time(content_rect.center().y + delta_screen.y)
                    - screen_y_to_time(content_rect.center().y);
                let delta_x = screen_x_to_x(content_rect.center().x + delta_screen.x)
                    - screen_x_to_x(content_rect.center().x);

                // 应用移动（作为preedit）
                for (i, &note_idx) in state.selected_notes.iter().enumerate() {
                    if let Some(&(orig_time, orig_x)) = drag_state.original_positions.get(i) {
                        let new_time = snapping_context
                            .snap_time(orig_time + delta_time)
                            .clamp(TIME_MIN, TIME_MAX);
                        let new_x = snapping_context
                            .snap_value(orig_x + delta_x)
                            .clamp(X_MIN, X_MAX);

                        if let Some(note) = chart_mut.layout_notes.get_mut(note_idx) {
                            note.time = new_time;
                            note.x = new_x;
                        }
                    }
                }
            }
        }
    }

    // 处理拖拽结束
    if content_response.drag_stopped_by(egui::PointerButton::Primary) {
        if let Some(box_state) = state.box_select_state.take() {
            // 完成框选
            let rect = Rect::from_two_pos(box_state.start_pos, box_state.current_pos);
            let selected: Vec<usize> = chart_mut
                .layout_notes
                .iter()
                .enumerate()
                .filter(|(_, note)| {
                    let pos = pos2(x_to_screen_x(note.x), time_to_screen_y(note.time));
                    rect.contains(pos)
                })
                .map(|(idx, _)| idx)
                .collect();

            if ui.input(|i| i.modifiers.ctrl) {
                // Ctrl：添加到选择
                for idx in selected {
                    if !state.selected_notes.contains(&idx) {
                        state.selected_notes.push(idx);
                    }
                }
            } else {
                state.selected_notes = selected;
            }
        } else if let Some(drag_state) = state.drag_state.take() {
            // 完成拖拽 - 提交命令
            let mut commands = Vec::new();
            for (i, &note_idx) in state.selected_notes.iter().enumerate() {
                if let Some(&(orig_time, orig_x)) = drag_state.original_positions.get(i) {
                    if let Some(note) = chart_mut.layout_notes.get(note_idx) {
                        if note.time != orig_time || note.x != orig_x {
                            // 先恢复原始位置
                            let current_time = note.time;
                            let current_x = note.x;

                            // 恢复
                            if let Some(n) = chart_mut.layout_notes.get_mut(note_idx) {
                                n.time = orig_time;
                                n.x = orig_x;
                            }

                            // 添加命令
                            commands.push(rizlium_chart::editing::commands::MoveLayoutNote {
                                new_time: current_time,
                                new_x: current_x,
                                note_path: rizlium_chart::editing::LayoutNotePath::new(note_idx),
                            });
                        }
                    }
                }
            }

            if !commands.is_empty() {
                use rizlium_chart::editing::commands::CommandSequence;
                let sequence = CommandSequence {
                    commands: commands.into_iter().map(|c| c.into()).collect(),
                    description: "Move layout notes".into(),
                };
                if let Err(err) = chart_edit_history.push(sequence, chart_mut) {
                    toast.error(err.to_string());
                }
            }
        }
    }

    // 处理删除键
    if ui.input(|i| i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace)) {
        if !state.selected_notes.is_empty() {
            // 从后往前删除以保持索引有效
            let mut sorted_indices = state.selected_notes.clone();
            sorted_indices.sort_by(|a, b| b.cmp(a));

            let commands: Vec<_> = sorted_indices
                .iter()
                .map(|&idx| {
                    rizlium_chart::editing::commands::RemoveLayoutNote {
                        note_path: rizlium_chart::editing::LayoutNotePath::new(idx),
                    }
                    .into()
                })
                .collect();

            let sequence = rizlium_chart::editing::commands::CommandSequence {
                commands,
                description: "Remove layout notes".into(),
            };

            if let Err(err) = chart_edit_history.push(sequence, chart_mut) {
                toast.error(err.to_string());
            }
            state.selected_notes.clear();
        }
    }

    // 处理滚动缩放
    if content_response.hovered() {
        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        let mut new_time_range = time_range.clone();
        let mut new_x_range = x_range.clone();

        if scroll_delta.y != 0.0 {
            if ui.input(|i| i.modifiers.ctrl) {
                // Ctrl+滚轮：缩放时间
                let zoom_factor = 1.0 + scroll_delta.y / 500.0;
                let center = (*new_time_range.start() + *new_time_range.end()) / 2.0;
                let half_range =
                    (new_time_range.end() - new_time_range.start()) / 2.0 / zoom_factor;
                new_time_range =
                    (center - half_range).max(TIME_MIN)..=(center + half_range).min(TIME_MAX);
            } else {
                // 滚轮：滚动时间
                let delta = -scroll_delta.y / 50.0;
                let range = new_time_range.end() - new_time_range.start();
                let new_start = (*new_time_range.start() + delta).clamp(TIME_MIN, TIME_MAX - range);
                new_time_range = new_start..=(new_start + range);
            }
        }

        if scroll_delta.x != 0.0 {
            // 水平滚动
            let delta = scroll_delta.x;
            let range = new_x_range.end() - new_x_range.start();
            let new_start = (*new_x_range.start() - delta).clamp(X_MIN, X_MAX - range);
            new_x_range = new_start..=(new_start + range);
        }

        state.time_range = Some(new_time_range);
        state.x_range = Some(new_x_range);
    }

    // 跟随游标
    if state.follow_cursor {
        let time_range = state.time_range.clone().unwrap_or(0.0..=10.0);
        let view_height = time_range.end() - time_range.start();
        let target_start = cursor_time - view_height / 2.0;
        let new_start = target_start.clamp(TIME_MIN, TIME_MAX - view_height);
        state.time_range = Some(new_start..=(new_start + view_height));
    }

    // 点击时间轴进行seek
    let timeline_response = ui.interact(timeline_rect, Id::new("layout_timeline"), Sense::click());
    if timeline_response.clicked() {
        if let Some(pos) = timeline_response.interact_pointer_pos() {
            let seek_time = screen_y_to_time(pos.y);
            time_control.write(TimeControlEvent::Seek(seek_time));
        }
    }

    // 分配UI空间
    ui.allocate_rect(available_rect, Sense::hover());
}
