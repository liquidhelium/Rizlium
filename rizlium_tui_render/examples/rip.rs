use std::{
    error::Error,
    fs::File,
    io::Read,
    io::{self, Stdout},
    time::{Duration, Instant},
};

use anyhow::Context;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use rizlium_chart::{chart::Chart, prelude::RizlineChart};
use rizlium_tui_render::{chart_cache, RizlineRender};

/// Load a Rizline chart from a packed zip bundle (e.g. `RIP.eicateve.0.zip`).
pub fn load_rizline_chart_from_bundle() -> anyhow::Result<Chart> {
    let mut chart_file = File::open("assets/RIP.eicateve.0/Chart_HD.json").context("read chart")?;
    let mut chart_text = String::new();
    chart_file
        .read_to_string(&mut chart_text)
        .context("read chart json")?;
    let rizline_chart: RizlineChart =
        serde_json::from_str(&chart_text).context("parse chart json")?;
    let chart: Chart = rizline_chart.try_into().context("convert chart")?;
    Ok(chart)
}
fn main() -> Result<(), Box<dyn Error>> {
    let chart = load_rizline_chart_from_bundle()?;
    let cache = chart_cache(&chart);

    let mut terminal = setup_terminal()?;
    let mut last_tick = Instant::now();
    let mut real_time = 0.0_f32;

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let game_time = cache.map_time(real_time);
            let widget = RizlineRender::new(&chart, &cache, game_time);
            frame.render_widget(widget, area);
        })?;

        let timeout = Duration::from_millis(16);
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('r') => {
                        real_time = 0.0;
                        last_tick = Instant::now();
                    }
                    _ => {}
                }
            }
        }

        let now = Instant::now();
        let dt = now.duration_since(last_tick);
        last_tick = now;
        real_time += dt.as_secs_f32();
    }

    restore_terminal(terminal)?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(
    mut terminal: Terminal<CrosstermBackend<Stdout>>,
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
