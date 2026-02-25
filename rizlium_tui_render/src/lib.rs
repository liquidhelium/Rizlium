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

const EPS: f32 = 1.0e-6;

/// Render configuration for the TUI widget.
#[derive(Debug, Clone)]
pub struct RizlineRenderConfig {
    pub line_glyph: char,
    pub note_glyph: char,
    pub hold_head_glyph: char,
    pub hold_tail_glyph: char,
    pub hold_body_glyph: char,
    pub mask_gradient_height_ratio: f32,
    pub top_mask_height_ratio: f32,
    pub show_line: Option<usize>,
}

impl Default for RizlineRenderConfig {
    fn default() -> Self {
        Self {
            line_glyph: '·',
            note_glyph: '●',
            hold_head_glyph: '●',
            hold_tail_glyph: '■',
            hold_body_glyph: '│',
            mask_gradient_height_ratio: GRADIENT_NORMALIZED_HEIGHT,
            top_mask_height_ratio: TOP_MASK_HEIGHT,
            show_line: None,
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

        fill_rect(buf, playfield.rect, background.to_tui());

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

        let mut render_ctx = RenderContext {
            playfield,
            chart: self.chart,
            cache: self.cache,
            game_time: self.game_time,
            cam_move,
            cam_scale,
            background,
            config: self.config.clone(),
        };

        render_lines(buf, &mut render_ctx);
        render_notes(buf, &mut render_ctx);
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
        let ny = (y - VIEW_RECT[0][1]) / VIEW_HEIGHT;

        if !nx.is_finite() || !ny.is_finite() {
            return None;
        }

        let width_units = (self.width * 0.5).max(1.0);
        let px_units = (nx * (width_units - 1.0)).round() as i32;
        let py = ((1.0 - ny) * (self.height - 1.0)).round() as i32;

        let x = self.rect.x as i32 + px_units * 2;
        let y = self.rect.y as i32 + py;

        if x < self.rect.x as i32
            || x >= (self.rect.x + self.rect.width) as i32
            || y < self.rect.y as i32
            || y >= (self.rect.y + self.rect.height) as i32
        {
            None
        } else {
            Some((x, y))
        }
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
        let alpha = alpha.clamp(0.0, 1.0);
        let r = self.r + (top.r - self.r) * alpha;
        let g = self.g + (top.g - self.g) * alpha;
        let b = self.b + (top.b - self.b) * alpha;
        let a = self.a + (top.a - self.a) * alpha;
        Rgba { r, g, b, a }
    }

    fn to_tui(self) -> Color {
        let r = (self.r.clamp(0.0, 1.0) * 255.0) as u8;
        let g = (self.g.clamp(0.0, 1.0) * 255.0) as u8;
        let b = (self.b.clamp(0.0, 1.0) * 255.0) as u8;
        Color::Rgb(r, g, b)
    }
}

impl From<ColorRGBA> for Rgba {
    fn from(value: ColorRGBA) -> Self {
        Self {
            r: value.r,
            g: value.g,
            b: value.b,
            a: value.a,
        }
    }
}

fn fill_rect(buf: &mut Buffer, rect: Rect, color: Color) {
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char(' ');
                cell.set_bg(color);
            }
        }
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

    for (line_idx, line) in chart.lines.iter().enumerate() {
        if let Some(show) = ctx.config.show_line {
            if show != line_idx {
                continue;
            }
        }

        for keypoint_idx in 0..line.points.points().len().saturating_sub(1) {
            let p1 = &line.points.points()[keypoint_idx];

            let pos1 = chart
                .with_cache(ctx.cache)
                .pos_for_linepoint_at(line_idx, keypoint_idx, ctx.game_time);
            let pos2 = chart
                .with_cache(ctx.cache)
                .pos_for_linepoint_at(line_idx, keypoint_idx + 1, ctx.game_time);

            let (Some(mut pos1), Some(mut pos2)) = (pos1, pos2) else {
                continue;
            };

            pos1[0] = (pos1[0] - ctx.cam_move) * ctx.cam_scale;
            pos1[1] *= ctx.cam_scale;
            pos2[0] = (pos2[0] - ctx.cam_move) * ctx.cam_scale;
            pos2[1] *= ctx.cam_scale;

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
    ctx: &RenderContext<'_>,
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
        draw_gradient_line(buf, ctx, a, b, color1, color2, acc / total_len, seg_len / total_len);
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

fn draw_gradient_line(
    buf: &mut Buffer,
    ctx: &RenderContext<'_>,
    a: [f32; 2],
    b: [f32; 2],
    c1: Rgba,
    c2: Rgba,
    start_t: f32,
    length_t: f32,
) {
    let Some((x0, y0)) = ctx.playfield.world_to_cell(a[0], a[1]) else {
        return;
    };
    let Some((x1, y1)) = ctx.playfield.world_to_cell(b[0], b[1]) else {
        return;
    };

    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let steps = dx.max(dy).max(1) as i32;

    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let x = x0 + ((x1 - x0) as f32 * t).round() as i32;
        let y = y0 + ((y1 - y0) as f32 * t).round() as i32;
        let world_y = a[1] + (b[1] - a[1]) * t;
        let mask_alpha = mask_alpha(ctx, world_y);

        let line_t = (start_t + length_t * t).clamp(0.0, 1.0);
        let line_color = c1.blend(c2, line_t);

        let effective_alpha = (mask_alpha * line_color.a).clamp(0.0, 1.0);
        let final_color = ctx.background.blend(line_color, effective_alpha);

        if let Some(cell) = get_cell_mut(buf, x, y, ctx.playfield.rect) {
            cell.set_char(ctx.config.line_glyph);
            cell.set_fg(final_color.to_tui());
            cell.set_bg(ctx.background.to_tui());
        }
    }
}

fn render_notes(buf: &mut Buffer, ctx: &mut RenderContext<'_>) {
    let chart = ctx.chart;
    let cache = ctx.cache;

    for (line_idx, line) in chart.lines.iter().enumerate() {
        for note in line.notes.iter() {
            let (note_time, hold_end) = match note.kind {
                NoteKind::Hold { end } => {
                    if end < ctx.game_time {
                        continue;
                    }
                    (note.time.max(ctx.game_time), Some(end))
                }
                _ => (note.time, None),
            };

            let pos = chart
                .with_cache(cache)
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
                let end_pos = chart
                    .with_cache(cache)
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

fn draw_note_glyph(
    buf: &mut Buffer,
    ctx: &RenderContext<'_>,
    pos: [f32; 2],
    color: Rgba,
    glyph: char,
) {
    let Some((x, y)) = ctx.playfield.world_to_cell(pos[0], pos[1]) else {
        return;
    };
    let mask_alpha = mask_alpha(ctx, pos[1]);
    let effective_alpha = (mask_alpha * color.a).clamp(0.0, 1.0);
    let final_color = ctx.background.blend(color, effective_alpha);

    if let Some(cell) = get_cell_mut(buf, x, y, ctx.playfield.rect) {
        cell.set_char(glyph);
        cell.set_fg(final_color.to_tui());
        cell.set_bg(ctx.background.to_tui());
    }
}

fn draw_hold_body(buf: &mut Buffer, ctx: &RenderContext<'_>, a: [f32; 2], b: [f32; 2], color: Rgba) {
    let Some((x0, y0)) = ctx.playfield.world_to_cell(a[0], a[1]) else {
        return;
    };
    let Some((x1, y1)) = ctx.playfield.world_to_cell(b[0], b[1]) else {
        return;
    };

    if x0 != x1 {
        draw_gradient_line(
            buf,
            ctx,
            a,
            b,
            color,
            color,
            0.0,
            1.0,
        );
        return;
    }

    let (start, end) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
    for y in start..=end {
        let world_y = a[1] + (b[1] - a[1]) * ((y - start) as f32 / (end - start).max(1) as f32);
        let mask_alpha = mask_alpha(ctx, world_y);
        let effective_alpha = (mask_alpha * color.a).clamp(0.0, 1.0);
        let final_color = ctx.background.blend(color, effective_alpha);

        if let Some(cell) = get_cell_mut(buf, x0, y, ctx.playfield.rect) {
            cell.set_char(ctx.config.hold_body_glyph);
            cell.set_fg(final_color.to_tui());
            cell.set_bg(ctx.background.to_tui());
        }
    }
}

fn mask_alpha(ctx: &RenderContext<'_>, world_y: f32) -> f32 {
    let h = VIEW_HEIGHT * ctx.cam_scale;
    let gradient = h * ctx.config.mask_gradient_height_ratio.max(EPS);
    let top_mask = h * ctx.config.top_mask_height_ratio.max(0.0);

    if world_y < 0.0 {
        return 0.0;
    }

    let bottom_alpha = if world_y <= gradient {
        (world_y / gradient.max(EPS)).clamp(0.0, 1.0)
    } else {
        1.0
    };

    // top fade: y in [h - top_mask, h - top_mask + gradient] -> alpha in [1, 0]
    let top_start = h - top_mask;
    let top_end = top_start + gradient;

    let top_alpha = if world_y >= top_start && world_y <= top_end {
        let t = (world_y - top_start) / gradient.max(EPS);
        (1.0 - t).clamp(0.0, 1.0)
    } else if world_y > top_end {
        0.0
    } else {
        1.0
    };

    (bottom_alpha * top_alpha).clamp(0.0, 1.0)
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
