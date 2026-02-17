mod app;
mod event;
mod ui;

use crate::app::{App, FileReport};
use anyhow::{bail, Context, Result};
use clap::Parser;
use crossterm::event::{self as ct_event, Event, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "ratunit",
    about = "A rat-powered TUI viewer for JUnit XML test reports"
)]
struct Cli {
    /// Path to a JUnit XML file or a directory containing XML files
    path: PathBuf,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let path = &cli.path;

    if !path.exists() {
        bail!("Path does not exist: {}", path.display());
    }

    let files = if path.is_dir() {
        let parsed = junit_parser::parse_directory(path)
            .with_context(|| format!("Failed to parse directory: {}", path.display()))?;
        if parsed.is_empty() {
            bail!("No XML files found in: {}", path.display());
        }
        parsed
            .into_iter()
            .map(|(name, data)| FileReport {
                filename: name,
                data,
            })
            .collect()
    } else {
        let data = junit_parser::parse_file(path)
            .with_context(|| format!("Failed to parse file: {}", path.display()))?;
        let filename = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());
        vec![FileReport { filename, data }]
    };

    let app = App::new(files);

    install_panic_hook();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_loop(&mut terminal, app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result
}

fn run_loop(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>, mut app: App) -> Result<()> {
    loop {
        terminal.draw(|frame| ui::render(frame, &app))?;

        if let Event::Key(key) = ct_event::read()? {
            if key.kind == KeyEventKind::Press {
                event::handle_key(&mut app, key);
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        original_hook(panic_info);
    }));
}
