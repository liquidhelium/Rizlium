use std::sync::{Arc, Mutex};
use std::time::Instant;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
    widgets::Widget,
};
use rizlium_chart::{
    chart::{
        Chart, ChartCache, ColorRGBA, EasingId, KeyPoint, Line, LinePointData, NoteKind,
        ThemeColor, ThemeData, ThemeTransition, Tween,
    },
    VIEW_RECT,
};

const VIEW_WIDTH: f32 = VIEW_RECT[1][0] - VIEW_RECT[0][0];
const VIEW_HEIGHT: f32 = VIEW_RECT[1][1] - VIEW_RECT[0][1];

const ASPECT_W: f32 = 9.0;
const ASPECT_H: f32 = 16.0;

const GRADIENT_NORMALIZED_HEIGHT: f32 = 0.05;
const TOP_MASK_HEIGHT: f32 = 0.2;
const RING_OFFSET: f32 = 0.2;
const RING_RADIUS: f32 = 43.0;

const EPS: f32 = 1.0e-6;

#[derive(Debug, Default, Clone)]
pub struct RenderStats {
    pub fill_ms: f32,
    pub lines_ms: f32,
    pub notes_ms: f32,
    pub rings_ms: f32,
}

/// Render configuration for the TUI widget.
#[derive(Debug, Clone)]
pub struct RizlineRenderConfig {
    pub note_glyph: char,
    pub hold_head_glyph: char,
    pub hold_tail_glyph: char,
    pub hold_body_glyph: char,
    pub mask_gradient_height_ratio: f32,
    pub top_mask_height_ratio: f32,
    pub show_line: Option<usize>,
    pub show_rings: bool,
    /// Sampling stride for braille dot drawing. Higher values reduce output volume.
    pub braille_step: usize,
    /// Cap for per-segment braille sampling steps.
    pub max_braille_steps: usize,
    /// Ring segment count.
    pub ring_steps: usize,
    /// Whether to clear the background each frame.
    pub clear_background: bool,
    /// Stride for background fill to reduce output (1 = full fill).
    pub background_fill_step: u16,

    /// Optional note render time window: (backward, forward) in seconds.
    pub note_time_window: Option<(f32, f32)>,
    /// Optional render timing stats sink.
    pub stats: Option<Arc<Mutex<RenderStats>>>,
}

impl Default for RizlineRenderConfig {
    fn default() -> Self {
        Self {
            note_glyph: '●',
            hold_head_glyph: '●',
            hold_tail_glyph: '■',
            hold_body_glyph: '│',
            mask_gradient_height_ratio: GRADIENT_NORMALIZED_HEIGHT,
            top_mask_height_ratio: TOP_MASK_HEIGHT,
            show_line: None,
            show_rings: true,
            braille_step: 1,
            max_braille_steps: 5000,
            ring_steps: 16,
            clear_background: true,
            background_fill_step: 1,
            note_time_window: None,
            stats: None,
        }
    }
}

/// A custom widget that renders a Rizline chart in a 9:16 playfield.
pub struct RizlineRender<'a> {
    chart: &'a Chart,
    cache: &'a ChartCache,
    game_time: f32,
    config: RizlineRenderConfig,
}

impl<'a> RizlineRender<'a> {
    pub fn new(chart: &'a Chart, cache: &'a ChartCache, game_time: f32) -> Self {
        Self {
            chart,
            cache,
            game_time,
            config: RizlineRenderConfig::default(),
        }
    }

    pub fn config(mut self, config: RizlineRenderConfig) -> Self {
        self.config = config;
        self
    }
}

impl Widget for RizlineRender<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let playfield = Playfield::fit(area);
        if playfield.rect.width == 0 || playfield.rect.height == 0 {
            return;
        }

        let theme = current_theme(self.chart, self.game_time);
        let background = Rgba::from(theme.color.background);

        if let Some(stats) = self.config.stats.as_ref() {
            if let Ok(mut stats) = stats.lock() {
                *stats = RenderStats::default();
            }
        }

        if self.config.clear_background {
            let start = Instant::now();
            fill_rect(
                buf,
                playfield.rect,
                background.to_tui(),
                self.config.background_fill_step,
            );
            if let Some(stats) = self.config.stats.as_ref() {
                if let Ok(mut stats) = stats.lock() {
                    stats.fill_ms = start.elapsed().as_secs_f32() * 1000.0;
                }
            }
        }

        let cam_move = self
            .chart
            .cam_move
            .value_padding(self.game_time)
            .unwrap_or(0.0);
        let cam_scale = self
            .chart
            .cam_scale
            .value_padding(self.game_time)
            .unwrap_or(1.0)
            .max(EPS);

        let top_world_y = playfield.cell_to_world_y(playfield.rect.y as i32) * cam_scale;
        let bottom_world_y = playfield
            .cell_to_world_y((playfield.rect.y + playfield.rect.height).saturating_sub(1) as i32)
            * cam_scale;
        let visible_world_y_min = bottom_world_y.min(top_world_y);
        let visible_world_y_max = bottom_world_y.max(top_world_y);

        let mut render_ctx = RenderContext {
            playfield,
            chart: self.chart,
            cache: self.cache,
            game_time: self.game_time,
            cam_move,
            cam_scale,
            background,
            config: self.config.clone(),
            mask_row_cache: vec![None; playfield.rect.height as usize],
            cell_alpha: vec![
                0.0;
                (playfield.rect.width as usize) * (playfield.rect.height as usize)
            ],
            visible_world_y_min,
            visible_world_y_max,
        };

        if render_ctx.config.show_rings {
            let start_rings = Instant::now();
            render_rings(buf, &mut render_ctx);
            if let Some(stats) = render_ctx.config.stats.as_ref() {
                if let Ok(mut stats) = stats.lock() {
                    stats.rings_ms = start_rings.elapsed().as_secs_f32() * 1000.0;
                }
            }
        }

        let start_lines = Instant::now();
        render_lines(buf, &mut render_ctx);
        if let Some(stats) = render_ctx.config.stats.as_ref() {
            if let Ok(mut stats) = stats.lock() {
                stats.lines_ms = start_lines.elapsed().as_secs_f32() * 1000.0;
            }
        }

        let start_notes = Instant::now();
        render_notes(buf, &mut render_ctx);
        if let Some(stats) = render_ctx.config.stats.as_ref() {
            if let Ok(mut stats) = stats.lock() {
                stats.notes_ms = start_notes.elapsed().as_secs_f32() * 1000.0;
            }
        }
    }
}

/// Build a cache for a chart, matching `rizlium_render` behavior.
pub fn chart_cache(chart: &Chart) -> ChartCache {
    ChartCache::from_chart(chart)
}
struct RenderContext<'a> {
    playfield: Playfield,
    chart: &'a Chart,
    cache: &'a ChartCache,
    game_time: f32,
    cam_move: f32,
    cam_scale: f32,
    background: Rgba,
    config: RizlineRenderConfig,
    mask_row_cache: Vec<Option<f32>>,
    cell_alpha: Vec<f32>,
    visible_world_y_min: f32,
    visible_world_y_max: f32,
}

#[derive(Clone, Copy)]
struct Playfield {
    rect: Rect,
    width: f32,
    height: f32,
}

impl Playfield {
    fn fit(area: Rect) -> Self {
        let target_ratio = ASPECT_W / ASPECT_H;
        let area_w_units = area.width as f32 * 0.5;
        let area_h_units = area.height as f32;
        if area_w_units <= 0.0 || area_h_units <= 0.0 {
            return Self {
                rect: area,
                width: 0.0,
                height: 0.0,
            };
        }

        let mut w_units = area_w_units;
        let mut h_units = area_w_units / target_ratio;
        if h_units > area_h_units {
            h_units = area_h_units;
            w_units = area_h_units * target_ratio;
        }

        let w = (w_units * 2.0).floor().max(0.0) as u16;
        let h = h_units.floor().max(0.0) as u16;
        let x = area.x + (area.width.saturating_sub(w)) / 2;
        let y = area.y + (area.height.saturating_sub(h)) / 2;

        Self {
            rect: Rect { x, y, width: w, height: h },
            width: w as f32,
            height: h as f32,
        }
    }

    fn world_to_cell(&self, x: f32, y: f32) -> Option<(i32, i32)> {
        let nx = (x - VIEW_RECT[0][0]) / VIEW_WIDTH;
        let ny = (y / VIEW_HEIGHT) + RING_OFFSET;

        if !nx.is_finite() || !ny.is_finite() {
            return None;
        }

        let width_units = (self.width * 0.5).max(1.0);
        let px_units = (nx * (width_units - 1.0)).round() as i32;
        let py = ((1.0 - ny) * (self.height - 1.0)).round() as i32;

        let x = self.rect.x as i32 + px_units * 2;
        let y = self.rect.y as i32 + py;

        Some((x, y))
    }

    fn world_to_dot(&self, x: f32, y: f32) -> Option<(i32, i32)> {
        let nx = (x - VIEW_RECT[0][0]) / VIEW_WIDTH;
        let ny = (y / VIEW_HEIGHT) + RING_OFFSET;

        if !nx.is_finite() || !ny.is_finite() {
            return None;
        }

        let dot_width = self.width * 2.0;
        let dot_height = self.height * 4.0;

        let dx = (nx * (dot_width - 1.0)).round() as i32;
        let dy = ((1.0 - ny) * (dot_height - 1.0)).round() as i32;

        Some((dx, dy))
    }

    fn cell_to_world_y(&self, y: i32) -> f32 {
        let py = y - self.rect.y as i32;
        // ny = 1.0 - py / (H-1)
        let ny = 1.0 - (py as f32 / (self.height - 1.0).max(1.0));
        (ny - RING_OFFSET) * VIEW_HEIGHT
    }
}

#[derive(Clone, Copy, Debug)]
struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Rgba {
    fn blend(self, top: Rgba, alpha: f32) -> Rgba {
        let t = alpha.clamp(0.0, 1.0);
        let r = self.r + (top.r - self.r) * t;
        let g = self.g + (top.g - self.g) * t;
        let b = self.b + (top.b - self.b) * t;
        let a = self.a + (top.a - self.a) * t;
        Rgba { r, g, b, a }
    }

    fn scale_alpha(self, scale: f32) -> Rgba {
        let s = scale.clamp(0.0, 1.0);
        Rgba {
            r: self.r * s,
            g: self.g * s,
            b: self.b * s,
            a: self.a * s,
        }
    }

    fn to_tui(self) -> Color {
        if self.a <= EPS {
            return Color::Rgb(0, 0, 0);
        }
        let inv_a = 1.0 / self.a;
        let r = ((self.r * inv_a).clamp(0.0, 1.0) * 255.0) as u8;
        let g = ((self.g * inv_a).clamp(0.0, 1.0) * 255.0) as u8;
        let b = ((self.b * inv_a).clamp(0.0, 1.0) * 255.0) as u8;
        Color::Rgb(r, g, b)
    }
}

impl From<ColorRGBA> for Rgba {
    fn from(value: ColorRGBA) -> Self {
        let a = value.a;
        Self {
            r: value.r * a,
            g: value.g * a,
            b: value.b * a,
            a,
        }
    }
}

fn fill_rect(buf: &mut Buffer, rect: Rect, color: Color, step: u16) {
    let step = step.max(1);
    let mut y = rect.y;
    while y < rect.y + rect.height {
        let mut x = rect.x;
        while x < rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(color);
            }
            x = x.saturating_add(step);
        }
        y = y.saturating_add(step);
    }
}

fn current_theme(chart: &Chart, time: f32) -> ThemeData {
    match chart.theme_at(time) {
        Ok(ThemeTransition { this, next, progress }) => {
            let progress = if progress.is_finite() {
                progress.clamp(0.0, 1.0)
            } else {
                0.0
            };
            <ThemeData as Tween>::lerp(*this, *next, progress)
        }
        Err(_) => ThemeData {
            color: ThemeColor {
                background: ColorRGBA::BLACK,
                note: ColorRGBA::WHITE,
                fx: ColorRGBA::WHITE,
            },
            is_challenge: false,
        },
    }
}

fn line_color_at(line: &Line, keypoint_idx: usize, time: f32) -> Rgba {
    let point_color = line
        .points
        .points()
        .get(keypoint_idx)
        .map(|p| p.relevant.color)
        .unwrap_or(ColorRGBA::WHITE);
    let line_color = line.line_color.value_padding(time).unwrap_or_default();
    Rgba::from(point_color + line_color)
}

fn render_lines(buf: &mut Buffer, ctx: &mut RenderContext<'_>) {
    let chart = ctx.chart;
    let chart_with_cache = chart.with_cache(ctx.cache);

    for (line_idx, line) in chart.lines.iter().enumerate() {
        if let Some(show) = ctx.config.show_line {
            if show != line_idx {
                continue;
            }
        }

        for keypoint_idx in 0..line.points.points().len().saturating_sub(1) {
            let p1 = &line.points.points()[keypoint_idx];

            let pos1 = chart_with_cache
                .pos_for_linepoint_at(line_idx, keypoint_idx, ctx.game_time);
            let pos2 = chart_with_cache
                .pos_for_linepoint_at(line_idx, keypoint_idx + 1, ctx.game_time);

            let (Some(mut pos1), Some(mut pos2)) = (pos1, pos2) else {
                continue;
            };

            pos1[0] = (pos1[0] - ctx.cam_move) * ctx.cam_scale;
            pos1[1] *= ctx.cam_scale;
            pos2[0] = (pos2[0] - ctx.cam_move) * ctx.cam_scale;
            pos2[1] *= ctx.cam_scale;

            let seg_min_y = pos1[1].min(pos2[1]);
            let seg_max_y = pos1[1].max(pos2[1]);
            if seg_max_y < ctx.visible_world_y_min || seg_min_y > ctx.visible_world_y_max {
                continue;
            }

            let color1 = line_color_at(line, keypoint_idx, ctx.game_time);
            let color2 = line_color_at(line, keypoint_idx + 1, ctx.game_time);

            let samples = sample_line_points(p1, pos1, pos2);
            draw_polyline(buf, ctx, &samples, color1, color2);
        }
    }
}

fn sample_line_points(
    p1: &KeyPoint<f32, LinePointData>,
    pos1: [f32; 2],
    pos2: [f32; 2],
) -> Vec<[f32; 2]> {
    let dx = (pos2[0] - pos1[0]).abs();
    let dy = (pos2[1] - pos1[1]).abs();

    let need_curve = p1.ease_type != EasingId::Linear && dx > EPS && dy > EPS;
    if !need_curve {
        return vec![pos1, pos2];
    }

    let mut count = (dy / 5.0).floor().max(2.0) as usize;
    if count > 5000 {
        count = 5000;
    }

    let mut points = Vec::with_capacity(count + 1);
    points.push(pos1);

    for i in 1..count {
        let t = i as f32 / count as f32;
        let x = <f32 as Tween>::ease(pos1[0], pos2[0], t, p1.ease_type);
        let y = <f32 as Tween>::lerp(pos1[1], pos2[1], t);
        points.push([x, y]);
    }

    points.push(pos2);
    points
}

fn draw_polyline(
    buf: &mut Buffer,
    ctx: &mut RenderContext<'_>,
    points: &[[f32; 2]],
    color1: Rgba,
    color2: Rgba,
) {
    if points.len() < 2 {
        return;
    }

    let total_len = polyline_length(points).max(EPS);
    let mut acc = 0.0;
    for i in 0..points.len() - 1 {
        let a = points[i];
        let b = points[i + 1];
        let seg_len = ((b[0] - a[0]).powi(2) + (b[1] - a[1]).powi(2)).sqrt();
        draw_braille_gradient_line(
            buf,
            ctx,
            a,
            b,
            color1,
            color2,
            acc / total_len,
            seg_len / total_len,
        );
        acc += seg_len;
    }
}

fn polyline_length(points: &[[f32; 2]]) -> f32 {
    points
        .windows(2)
        .map(|w| {
            let dx = w[1][0] - w[0][0];
            let dy = w[1][1] - w[0][1];
            (dx * dx + dy * dy).sqrt()
        })
        .sum()
}

fn color_to_rgba(color: Color, fallback: Rgba) -> Rgba {
    match color {
        Color::Rgb(r, g, b) => Rgba {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        },
        Color::Reset => fallback,
        _ => fallback,
    }
}

fn blend_cell_colors(
    cell: &mut ratatui::buffer::Cell,
    color: Rgba,
    background: Rgba,
) {
    let base_bg = color_to_rgba(cell.bg, background);
    let a = color.a.clamp(0.0, 1.0);
    let inv_a = 1.0 - a;

    let blended_fg = Rgba {
        r: color.r + base_bg.r * inv_a,
        g: color.g + base_bg.g * inv_a,
        b: color.b + base_bg.b * inv_a,
        a: 1.0,
    };

    cell.set_fg(blended_fg.to_tui());
}

fn set_braille_dot(
    buf: &mut Buffer,
    ctx: &mut RenderContext<'_>,
    x_dot: i32,
    y_dot: i32,
    color: Rgba,
) {
    let char_x = x_dot.div_euclid(2);
    let char_y = y_dot.div_euclid(4);
    let dot_x = x_dot.rem_euclid(2) as u16;
    let dot_y = y_dot.rem_euclid(4) as u16;

    let rect = ctx.playfield.rect;
    let x = rect.x as i32 + char_x;
    let y = rect.y as i32 + char_y;

    if x < rect.x as i32
        || x >= (rect.x + rect.width) as i32
        || y < rect.y as i32
        || y >= (rect.y + rect.height) as i32
    {
        return;
    }

    let should_update_color = should_write_cell(ctx, x, y, color.a);

    let x = x as u16;
    let y = y as u16;

    if let Some(cell) = buf.cell_mut((x, y)) {
        let mut bits = 0u8;
        let current_char = cell.symbol().chars().next().unwrap_or(' ');
        if (0x2800..=0x28FF).contains(&(current_char as u32)) {
            bits = (current_char as u32 - 0x2800) as u8;
        }

        let bit = match (dot_x, dot_y) {
            (0, 0) => 0,
            (0, 1) => 1,
            (0, 2) => 2,
            (1, 0) => 3,
            (1, 1) => 4,
            (1, 2) => 5,
            (0, 3) => 6,
            (1, 3) => 7,
            _ => 0,
        };

        bits |= 1 << bit;
        cell.set_char(char::from_u32(0x2800 + bits as u32).unwrap());
        if should_update_color {
            blend_cell_colors(cell, color, ctx.background);
        }
    }
}

fn draw_braille_gradient_line(
    buf: &mut Buffer,
    ctx: &mut RenderContext<'_>,
    a: [f32; 2],
    b: [f32; 2],
    c1: Rgba,
    c2: Rgba,
    start_t: f32,
    length_t: f32,
) {
    let Some((x0, y0)) = ctx.playfield.world_to_dot(a[0], a[1]) else {
        return;
    };
    let Some((x1, y1)) = ctx.playfield.world_to_dot(b[0], b[1]) else {
        return;
    };

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let max_steps = ctx.config.max_braille_steps.max(1).min(i32::MAX as usize) as i32;
    let steps = dx.max(dy).max(1).min(max_steps);
    let step = ctx.config.braille_step.max(1) as i32;

    let mut i = 0;
    while i <= steps {
        let t = i as f32 / steps as f32;
        let x = x0 + ((x1 - x0) as f32 * t).round() as i32;
        let y = y0 + ((y1 - y0) as f32 * t).round() as i32;

        let char_y_in_buf = ctx.playfield.rect.y as i32 + y.div_euclid(4);
        let mask_overlay = get_mask_overlay_cached(ctx, char_y_in_buf);

        let line_t = (start_t + length_t * t).clamp(0.0, 1.0);
        let mut line_color = c1.blend(c2, line_t);
        line_color = line_color.scale_alpha(1.0 - mask_overlay);
        if line_color.a <= EPS {
            i += step;
            continue;
        }

        set_braille_dot(
            buf,
            ctx,
            x,
            y,
            line_color,
        );
        i += step;
    }
}

fn render_notes(buf: &mut Buffer, ctx: &mut RenderContext<'_>) {
    let chart = ctx.chart;
    let chart_with_cache = chart.with_cache(ctx.cache);
    let (window_start, window_end) = ctx
        .config
        .note_time_window
        .map(|(backward, forward)| (ctx.game_time - backward, ctx.game_time + forward))
        .unwrap_or((f32::NEG_INFINITY, f32::INFINITY));

    for (line_idx, line) in chart.lines.iter().enumerate() {
        for note in line.notes.iter() {
            if note.time > window_end {
                break;
            }
            if note.time < window_start {
                if let NoteKind::Hold { end } = note.kind {
                    if end < window_start {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            let (note_time, hold_end) = match note.kind {
                NoteKind::Hold { end } => {
                    if end < ctx.game_time {
                        continue;
                    }
                    (note.time.max(ctx.game_time), Some(end))
                }
                _ => {
                    if note.time < ctx.game_time - 0.1 { // Small window to keep hit notes visible briefly
                        continue;
                    }
                    (note.time, None)
                }
            };

            let pos = chart_with_cache
                .line_pos_at_clamped(line_idx, note_time, ctx.game_time);
            let Some(mut pos) = pos else { continue };
            pos[0] = (pos[0] - ctx.cam_move) * ctx.cam_scale;
            pos[1] *= ctx.cam_scale;

            let note_theme = current_theme(chart, note.time);
            let note_color = Rgba::from(note_theme.color.note);
            let head_glyph = if hold_end.is_some() {
                ctx.config.hold_head_glyph
            } else {
                ctx.config.note_glyph
            };

            draw_note_glyph(buf, ctx, pos, note_color, head_glyph);

            if let Some(end) = hold_end {
                let end_pos = chart_with_cache
                    .line_pos_at_clamped(line_idx, end, ctx.game_time);
                let Some(mut end_pos) = end_pos else { continue };
                end_pos[0] = (end_pos[0] - ctx.cam_move) * ctx.cam_scale;
                end_pos[1] *= ctx.cam_scale;

                draw_note_glyph(buf, ctx, end_pos, note_color, ctx.config.hold_tail_glyph);
                draw_hold_body(buf, ctx, pos, end_pos, note_color);
            }
        }
    }
}

fn render_rings(buf: &mut Buffer, ctx: &mut RenderContext<'_>) {
    let chart = ctx.chart;
    let chart_with_cache = chart.with_cache(ctx.cache);

    for (line_idx, line) in chart.lines.iter().enumerate() {
        let Some(mut pos) = chart_with_cache.line_pos_at(line_idx, ctx.game_time, ctx.game_time) else {
            continue;
        };

        pos[0] = (pos[0] - ctx.cam_move) * ctx.cam_scale;
        pos[1] *= ctx.cam_scale;

        let ring_color_rgba = line.ring_color.value_padding(ctx.game_time).unwrap_or_default();
        let ring_color = Rgba::from(ring_color_rgba);
        if ring_color.a <= EPS {
            continue;
        }

        draw_ring(buf, ctx, pos, ring_color);
    }
}

fn draw_ring(buf: &mut Buffer, ctx: &mut RenderContext<'_>, pos: [f32; 2], color: Rgba) {
    let radius = RING_RADIUS; // From rizlium_render/src/rings.rs
    let Some((cx, cy)) = ctx.playfield.world_to_dot(pos[0], pos[1]) else {
        return;
    };

    let dot_width = (ctx.playfield.width * 2.0).max(1.0);
    let dot_height = (ctx.playfield.height * 4.0).max(1.0);
    let rx = ((radius / VIEW_WIDTH) * (dot_width - 1.0)).round() as i32;
    let ry = ((radius / VIEW_HEIGHT) * (dot_height - 1.0)).round() as i32;
    let r = rx.min(ry).max(1);

    draw_ellipse_dots(buf, ctx, cx, cy, r, r, color);
}

fn draw_ellipse_dots(
    buf: &mut Buffer,
    ctx: &mut RenderContext<'_>,
    cx: i32,
    cy: i32,
    rx: i32,
    ry: i32,
    color: Rgba,
) {
    let rx = rx as i64;
    let ry = ry as i64;
    let mut x: i64 = 0;
    let mut y: i64 = ry;

    let rx2 = rx * rx;
    let ry2 = ry * ry;

    let mut dx = 2 * ry2 * x;
    let mut dy = 2 * rx2 * y;

    let mut p1: i64 = ry2 - rx2 * ry + rx2 / 4;

    while dx < dy {
        plot_ellipse_points(buf, ctx, cx, cy, x as i32, y as i32, color);

        x += 1;
        dx += 2 * ry2;

        if p1 < 0 {
            p1 += ry2 + dx;
        } else {
            y -= 1;
            dy -= 2 * rx2;
            p1 += ry2 + dx - dy;
        }
    }

    let mut p2: f64 = (ry2 as f64) * ((x as f64 + 0.5).powi(2))
        + (rx2 as f64) * ((y as f64 - 1.0).powi(2))
        - (rx2 as f64) * (ry2 as f64);

    while y >= 0 {
        plot_ellipse_points(buf, ctx, cx, cy, x as i32, y as i32, color);

        if p2 > 0.0 {
            y -= 1;
            dy -= 2 * rx2;
            p2 += (rx2 as f64) - (dy as f64);
        } else {
            x += 1;
            dx += 2 * ry2;
            y -= 1;
            dy -= 2 * rx2;
            p2 += (rx2 as f64) - (dy as f64) + (dx as f64);
        }
    }
}

fn plot_ellipse_points(
    buf: &mut Buffer,
    ctx: &mut RenderContext<'_>,
    cx: i32,
    cy: i32,
    x: i32,
    y: i32,
    color: Rgba,
) {
    set_braille_dot(buf, ctx, cx + x, cy + y, color);
    set_braille_dot(buf, ctx, cx - x, cy + y, color);
    set_braille_dot(buf, ctx, cx + x, cy - y, color);
    set_braille_dot(buf, ctx, cx - x, cy - y, color);
}

fn draw_note_glyph(
    buf: &mut Buffer,
    ctx: &mut RenderContext<'_>,
    pos: [f32; 2],
    color: Rgba,
    glyph: char,
) {
    let Some((x, y)) = ctx.playfield.world_to_cell(pos[0], pos[1]) else {
        return;
    };
    let mask_overlay = get_mask_overlay_cached(ctx, y);
    let mut final_color = color;
    final_color = final_color.scale_alpha(1.0 - mask_overlay);
    if final_color.a <= EPS {
        return;
    }

    if !should_write_cell(ctx, x, y, final_color.a) {
        return;
    }

    if let Some(cell) = get_cell_mut(buf, x, y, ctx.playfield.rect) {
        cell.set_char(glyph);
        blend_cell_colors(cell, final_color, ctx.background);
    }
}

fn draw_hold_body(buf: &mut Buffer, ctx: &mut RenderContext<'_>, a: [f32; 2], b: [f32; 2], color: Rgba) {
    // Hold body should also respect mask
    draw_braille_gradient_line(
        buf,
        ctx,
        a,
        b,
        color,
        color,
        0.0,
        1.0,
    );
}

fn get_mask_overlay_cached(ctx: &mut RenderContext<'_>, cell_y: i32) -> f32 {
    let row = cell_y - ctx.playfield.rect.y as i32;
    if row < 0 || row >= ctx.playfield.rect.height as i32 {
        let world_y = ctx.playfield.cell_to_world_y(cell_y);
        return get_mask_overlay(ctx, world_y);
    }

    let idx = row as usize;
    if let Some(value) = ctx.mask_row_cache[idx] {
        return value;
    }

    let world_y = ctx.playfield.cell_to_world_y(cell_y);
    let value = get_mask_overlay(ctx, world_y);
    ctx.mask_row_cache[idx] = Some(value);
    value
}

fn get_mask_overlay(ctx: &RenderContext<'_>, world_y: f32) -> f32 {
    let h = VIEW_HEIGHT;
    let gradient_h = h * ctx.config.mask_gradient_height_ratio.max(EPS);
    let top_gradient_h = gradient_h * 2.0;
    let top_mask_h = h * ctx.config.top_mask_height_ratio.max(0.0);
    let viewport_top = (1.0 - RING_OFFSET) * h;
    let ring_clear_y = viewport_top;
    let top_mask_start = (viewport_top - top_mask_h).max(ring_clear_y);

    if world_y < 0.0 {
        // Below 0: it's either solid background or gradient
        if world_y < -gradient_h {
            1.0
        } else {
            let t = (world_y.abs() / gradient_h).clamp(0.0, 1.0);
            t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
        }
    } else if world_y > top_mask_start {
        // Above top threshold
        if world_y > top_mask_start + top_gradient_h {
            1.0
        } else {
            ((world_y - top_mask_start) / top_gradient_h).clamp(0.0, 1.0)
        }
    } else {
        0.0
    }
}

fn cell_alpha_index(rect: Rect, x: i32, y: i32) -> Option<usize> {
    if x < rect.x as i32
        || y < rect.y as i32
        || x >= (rect.x + rect.width) as i32
        || y >= (rect.y + rect.height) as i32
    {
        return None;
    }
    let local_x = (x - rect.x as i32) as usize;
    let local_y = (y - rect.y as i32) as usize;
    let width = rect.width as usize;
    Some(local_y * width + local_x)
}

fn should_write_cell(ctx: &mut RenderContext<'_>, x: i32, y: i32, new_alpha: f32) -> bool {
    let Some(idx) = cell_alpha_index(ctx.playfield.rect, x, y) else {
        return false;
    };
    let current = ctx.cell_alpha[idx];
    if new_alpha + EPS < current {
        return false;
    }
    ctx.cell_alpha[idx] = new_alpha;
    true
}

fn get_cell_mut(buf: &mut Buffer, x: i32, y: i32, rect: Rect) -> Option<&mut ratatui::buffer::Cell> {
    if x < rect.x as i32
        || y < rect.y as i32
        || x >= (rect.x + rect.width) as i32
        || y >= (rect.y + rect.height) as i32
    {
        return None;
    }
    buf.cell_mut((x as u16, y as u16))
}
