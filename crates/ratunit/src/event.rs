use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.should_quit = true;
        }

        KeyCode::Char('j') | KeyCode::Down => app.select_next(),
        KeyCode::Char('k') | KeyCode::Up => app.select_prev(),

        KeyCode::Char('g') | KeyCode::Home => app.select_first(),
        KeyCode::Char('G') | KeyCode::End => app.select_last(),

        KeyCode::PageDown => app.page_down(),
        KeyCode::PageUp => app.page_up(),

        KeyCode::Enter | KeyCode::Char('l') | KeyCode::Right => app.enter(),
        KeyCode::Esc | KeyCode::Char('h') | KeyCode::Left | KeyCode::Backspace => app.go_back(),

        KeyCode::Tab => app.next_file(),
        KeyCode::BackTab => app.prev_file(),

        _ => {}
    }
}
