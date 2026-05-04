mod app;
mod config;
mod error;
mod input;
mod library;
mod lyrics;
mod mpris;
mod player;
mod playlist;
mod theme;
mod ui;

use std::io;
use std::path::Path;
use std::time::Duration;

use crossterm::{
    event::{self, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::*;

use crate::app::App;

fn main() -> anyhow::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::cursor::Hide,
        crossterm::event::EnableMouseCapture
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // MPRIS D-Bus integration (runs in background thread)
    let mpris_shared = std::sync::Arc::new(crate::mpris::MprisShared::new());
    crate::mpris::start_mpris(mpris_shared.clone());

    // Create app
    let mut app = App::new(mpris_shared)?;

    // UI layout tracking (separate from app so terminal.draw can borrow both)
    let mut ui_layout = crate::app::UiLayout::default();

    // If music dirs are configured, scan them
    let music_dirs: Vec<String> = app.config.music_dirs.clone();
    let mut first_dir = true;
    for dir in &music_dirs {
        let path = Path::new(dir);
        if path.exists() {
            if first_dir {
                app.library.current_dir = path.to_path_buf();
                first_dir = false;
            }
            if let Err(e) = poll_scan(&mut app, path) {
                app.status_msg = format!("Scan error: {}", e);
                app.status_timer = 120;
            }
        }
    }

    // Main event loop
    let tick_rate = Duration::from_millis(16); // ~60fps
    loop {
        terminal.draw(|f| ui::render(f, &app, &mut ui_layout))?;

        if !app.running {
            break;
        }

        // Poll for events with timeout
        if event::poll(tick_rate)? {
            match event::read()? {
                Event::Key(key_event) => {
                    if let Err(e) = app.handle_event(&key_event) {
                        app.status_msg = format!("Error: {}", e);
                        app.status_timer = 120;
                    }
                }
                Event::Mouse(mouse_event) => {
                    if let Err(e) = app.handle_mouse(mouse_event, &ui_layout) {
                        app.status_msg = format!("Error: {}", e);
                        app.status_timer = 120;
                    }
                }
                _ => {}
            }
        }

        app.tick();
    }

    // Save state on exit
    app.save_state();

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show,
        crossterm::event::DisableMouseCapture
    )?;

    Ok(())
}

/// Synchronous directory scan (for simplicity).
fn poll_scan(app: &mut App, path: &Path) -> anyhow::Result<()> {
    let mut tracks = Vec::new();
    scan_dir(path, &mut tracks)?;

    for track in tracks {
        app.library.tracks.push(std::sync::Arc::new(track));
    }
    app.library.update_filter();

    Ok(())
}

fn scan_dir(dir: &Path, tracks: &mut Vec<crate::library::TrackInfo>) -> anyhow::Result<()> {
    let entries = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        if name.starts_with('.') {
            continue;
        }
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            scan_dir(&path, tracks)?;
        } else if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            if crate::library::SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
                if let Ok(info) = crate::library::MusicLibrary::read_metadata_file(&path) {
                    tracks.push(info);
                }
            }
        }
    }
    Ok(())
}
