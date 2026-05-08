mod app;
mod config;
mod engine;
mod events;
mod ui;

use std::{io, path::PathBuf};

use anyhow::Context;
use app::{mode_from_config, App, Mode};
use clap::{Parser, ValueEnum};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

#[derive(Debug, Clone, Parser)]
#[command(author, version, about)]
struct Cli {
    #[arg(short, long, value_enum)]
    mode: Option<CliMode>,
    #[arg(short, long)]
    words: Option<usize>,
    #[arg(short, long)]
    time: Option<u64>,
    #[arg(long)]
    config: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum CliMode {
    Words,
    Time,
    Quote,
}

impl From<CliMode> for Mode {
    fn from(value: CliMode) -> Self {
        match value {
            CliMode::Words => Mode::Words(25),
            CliMode::Time => Mode::Time(30),
            CliMode::Quote => Mode::Quote,
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = config::Config::load(cli.config.as_deref()).context("load config")?;
    let mode = match cli.mode {
        Some(CliMode::Words) => Mode::Words(cli.words.unwrap_or(config.test.word_count).max(1)),
        Some(CliMode::Time) => Mode::Time(cli.time.unwrap_or(config.test.duration).max(1)),
        Some(CliMode::Quote) => Mode::Quote,
        None => mode_from_config(&config),
    };

    let mut terminal = setup_terminal()?;
    let result = run_app(&mut terminal, App::new(config, mode));
    restore_terminal(&mut terminal)?;
    result.context("run app")
}

fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> io::Result<()> {
    let events = events::EventStream::spawn();
    terminal.draw(|frame| ui::render(frame, &app.state))?;
    while !app.should_quit() {
        match events.recv() {
            Ok(event) => app.handle_event(event),
            Err(_) => app.quit(),
        }
        terminal.draw(|frame| ui::render(frame, &app.state))?;
    }
    app.save_outputs();
    Ok(())
}
