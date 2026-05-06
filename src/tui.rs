use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::app::{App, AppMode, CategoryFocus, InputMode};
use crate::ui::draw_ui;

pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

pub(crate) fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub(crate) fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    Ok(())
}

/// Best-effort terminal cleanup used from the panic hook.
pub(crate) fn force_restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

pub(crate) fn run_app(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| draw_ui(f, app))?;

        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.screen {
                    AppMode::List => match app.mode {
                        InputMode::Normal => handle_list_normal_key(app, key.code),
                        InputMode::Inserting => handle_list_insert_key(app, key.code),
                    },
                    AppMode::CategoryEdit => handle_category_edit_key(app, key.code),
                }
            }
        }
    }
    Ok(())
}

fn handle_list_normal_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('a') => app.enter_insert_mode(),
        KeyCode::Char('d') => app.delete_selected(),
        KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('c') => app.enter_category_edit(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        KeyCode::Char(c) if ('1'..='9').contains(&c) => {
            let idx = (c as u8 - b'1') as usize;
            app.toggle_tag_on_selected(idx);
        }
        _ => {}
    }
}

fn handle_list_insert_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Enter => app.confirm_new_task(),
        KeyCode::Esc => app.cancel_insert_mode(),
        KeyCode::Backspace => {
            app.input_buffer.pop();
        }
        KeyCode::Char(c) => app.input_buffer.push(c),
        _ => {}
    }
}

fn handle_category_edit_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Esc => app.cancel_category_edit(),
        KeyCode::Enter => app.confirm_category_edit(),
        KeyCode::Tab => app.category_toggle_focus(),
        _ => match app.category_focus {
            CategoryFocus::Name => match code {
                KeyCode::Backspace => app.category_name_pop(),
                KeyCode::Char(c) => app.category_name_push(c),
                _ => {}
            },
            CategoryFocus::Color => match code {
                KeyCode::Left | KeyCode::Char('h') => app.category_color_prev(),
                KeyCode::Right | KeyCode::Char('l') => app.category_color_next(),
                _ => {}
            },
        },
    }
}
