//! # ftsim-tui
//!
//! This crate contains the main entry point and event loop for the terminal
//! user interface.

#![forbid(unsafe_code)]

use crate::app::App;
use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ftsim_engine::{control::ControlMsg, telemetry::snapshot::Snapshot};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    time::{Duration, Instant},
};

mod app;
mod input;
mod theme;
mod ui;

/// The main entry point for running the TUI.
/// It takes a receiver for `Snapshot` updates from the engine and a sender for control messages.
pub fn run_tui(
    snapshot_rx: crossbeam_channel::Receiver<Snapshot>,
    control_tx: crossbeam_channel::Sender<ControlMsg>,
) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and run the event loop
    let mut app = App::new(control_tx);
    let res = run_app(&mut terminal, &mut app, snapshot_rx);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("TUI Error: {:?}", err)
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    snapshot_rx: crossbeam_channel::Receiver<Snapshot>,
) -> io::Result<()> {
    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        // Handle input and updates
        if crossterm::event::poll(timeout)? {
            if let CEvent::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    return Ok(());
                }
                input::handle_key_press(key, app);
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }

        // Process all pending snapshots from the engine
        while let Ok(snapshot) = snapshot_rx.try_recv() {
            app.update_snapshot(snapshot);
        }
    }
}
