//! The real terminal: raw mode, alternate screen, mouse; restored on every way out.

use std::io;
use std::time::Instant;

use ratatui::crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind,
};
use ratatui::crossterm::execute;

use crate::app::{App, Flow};
use crate::ui::draw;

/// Leaves raw mode and the alternate screen when dropped, also on an error or a panic
/// that unwinds.
struct Restore;

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        ratatui::restore();
    }
}

/// Runs the editor until the user quits.
pub fn run(app: &mut App) -> io::Result<()> {
    let mut terminal = ratatui::try_init()?;
    let _restore = Restore;
    execute!(io::stdout(), EnableMouseCapture)?;

    // A panic prints its message after the screen is back, and releases the mouse too
    // (ratatui's own hook does the rest).
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(io::stdout(), DisableMouseCapture);
        previous(info);
    }));

    loop {
        terminal.draw(|frame| draw(frame, app))?;

        // With old pages, wait for an event only until they are due; then lay them out
        // again and draw (DN-02). A key never waits for the layout.
        if let Some(wait) = app.pages_wait(Instant::now()) {
            if !event::poll(wait)? {
                app.recompute_pages();
                continue;
            }
        }

        match event::read()? {
            Event::Key(key) if key.kind != KeyEventKind::Release => {
                if app.handle_key(key) == Flow::Quit {
                    return Ok(());
                }
            }
            Event::Mouse(mouse) => app.handle_mouse(mouse),
            _ => {}
        }
    }
}
