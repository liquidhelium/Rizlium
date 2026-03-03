use std::{
    error::Error,
    fs::File,
    io::Read,
    io::{self, BufWriter, Stdout},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::Context;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::Paragraph,
    Terminal,
};

use rizlium_chart::{chart::Chart, prelude::RizlineChart};
use rizlium_tui_render::{chart_cache, RenderStats, RizlineRender, RizlineRenderConfig};

/// Load a Rizline chart from a packed zip bundle (e.g. `RIP.eicateve.0.zip`).
pub fn load_rizline_chart_from_bundle() -> anyhow::Result<Chart> {
    let mut chart_file = File::open("assets/crystalized/Camellia0IN.json").context("read chart")?;
    let mut chart_text = String::new();
    chart_file
        .read_to_string(&mut chart_text)
        .context("read chart json")?;
    let rizline_chart: RizlineChart =
        serde_json::from_str(&chart_text).context("parse chart json")?;
    let chart: Chart = rizline_chart.try_into().context("convert chart")?;
    Ok(chart)
}
#[derive(Clone, Copy, Debug)]
enum RenderMode {
    Full,
    ChartOnly,
    StatsOnly,
}

fn main() -> Result<(), Box<dyn Error>> {
    let chart = load_rizline_chart_from_bundle()?;
    let cache = chart_cache(&chart);

    let mut terminal = setup_terminal()?;
    let mut last_tick = Instant::now();
    let mut real_time = 0.0_f32;

    let mut stats_last = Instant::now();
    let mut stats_frames: u32 = 0;
    let mut acc_draw = Duration::ZERO;
    let mut acc_frame = Duration::ZERO;
    let mut stats_text =
        String::from("Mode: Full | Quality: HQ | FPS: -- | draw: -- ms | frame: -- ms");
    let mut render_mode = RenderMode::Full;
    let mut render_config = RizlineRenderConfig::default();
    let stats_sink = Arc::new(Mutex::new(RenderStats::default()));
    render_config.stats = Some(stats_sink.clone());
    let mut quality_label = String::from("HQ");

    loop {
        let frame_start = Instant::now();
        let draw_start = Instant::now();
        terminal.draw(|frame| {
            let area = frame.area();
            match render_mode {
                RenderMode::Full => {
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints([Constraint::Length(1), Constraint::Min(0)])
                        .split(area);

                    let stats = Paragraph::new(stats_text.as_str());
                    frame.render_widget(stats, chunks[0]);

                    let game_time = cache.map_time(real_time);
                    let widget = RizlineRender::new(&chart, &cache, game_time)
                        .config(render_config.clone());
                    frame.render_widget(widget, chunks[1]);
                }
                RenderMode::ChartOnly => {
                    let game_time = cache.map_time(real_time);
                    let widget = RizlineRender::new(&chart, &cache, game_time)
                        .config(render_config.clone());
                    frame.render_widget(widget, area);
                }
                RenderMode::StatsOnly => {
                    let stats = Paragraph::new(stats_text.as_str());
                    frame.render_widget(stats, area);
                }
            }
        })?;
        let draw_dt = draw_start.elapsed();
        acc_draw += draw_dt;

        let timeout = Duration::from_millis(16);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        real_time = 0.0;
                        last_tick = Instant::now();
                    }
                    KeyCode::Char('1') => render_mode = RenderMode::Full,
                    KeyCode::Char('2') => render_mode = RenderMode::ChartOnly,
                    KeyCode::Char('3') => render_mode = RenderMode::StatsOnly,
                    KeyCode::Char('4') => {
                        render_config.braille_step = 4;
                        render_config.max_braille_steps = 600;
                        render_config.ring_steps = 6;
                        render_config.clear_background = false;
                        render_config.background_fill_step = 2;
                        render_config.note_set_bg = false;
                        render_config.note_time_window = Some((2.0, 5.0));
                        quality_label = String::from("LQ");
                    }
                    KeyCode::Char('5') => {
                        render_config = RizlineRenderConfig::default();
                        render_config.stats = Some(stats_sink.clone());
                        quality_label = String::from("HQ");
                    }
                    KeyCode::Char('6') => {
                        render_config.braille_step = 8;
                        render_config.max_braille_steps = 300;
                        render_config.ring_steps = 4;
                        render_config.clear_background = false;
                        render_config.background_fill_step = 4;
                        render_config.note_set_bg = false;
                        render_config.note_time_window = Some((1.0, 3.0));
                        quality_label = String::from("ULQ");
                    }
                    _ => {}
                }
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last_tick);
        last_tick = now;
        real_time += dt.as_secs_f32();

        let frame_dt = frame_start.elapsed();
        acc_frame += frame_dt;
        stats_frames += 1;

        if stats_last.elapsed() >= Duration::from_secs(1) && stats_frames > 0 {
            let elapsed = stats_last.elapsed().as_secs_f32();
            let fps = stats_frames as f32 / elapsed;
            let avg_draw_ms = acc_draw.as_secs_f32() * 1000.0 / stats_frames as f32;
            let avg_frame_ms = acc_frame.as_secs_f32() * 1000.0 / stats_frames as f32;
            let mode_label = match render_mode {
                RenderMode::Full => "Full",
                RenderMode::ChartOnly => "ChartOnly",
                RenderMode::StatsOnly => "StatsOnly",
            };
            let (fill_ms, lines_ms, notes_ms, rings_ms) = stats_sink
                .lock()
                .map(|s| (s.fill_ms, s.lines_ms, s.notes_ms, s.rings_ms))
                .unwrap_or((0.0, 0.0, 0.0, 0.0));
            stats_text = format!(
                "Mode: {} | Quality: {} | FPS: {:.1} | draw: {:.2}ms | frame: {:.2}ms | fill: {:.2} | lines: {:.2} | notes: {:.2} | rings: {:.2}",
                mode_label, quality_label, fps, avg_draw_ms, avg_frame_ms, fill_ms, lines_ms, notes_ms, rings_ms
            );
            stats_last = Instant::now();
            stats_frames = 0;
            acc_draw = Duration::ZERO;
            acc_frame = Duration::ZERO;
        }
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<BufWriter<Stdout>>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(BufWriter::new(stdout));
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<BufWriter<Stdout>>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}
