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

use crate::app::{App, AppMode, CategoryScreenMode, InputMode};
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
                    AppMode::CategoryEdit => handle_category_screen_key(app, key.code),
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
        KeyCode::Char('x') => app.toggle_active_selected(),
        KeyCode::Char('i') => app.toggle_show_inactive(),
        KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Char('c') => app.enter_category_edit(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        _ => {}
    }
}

fn handle_list_insert_key(app: &mut App, code: KeyCode) {
    if app.slash_menu.is_some() {
        match code {
            KeyCode::Up => app.slash_menu_select_prev(),
            KeyCode::Down | KeyCode::Tab => app.slash_menu_select_next(),
            KeyCode::Esc => app.slash_menu_close(),
            KeyCode::Enter => app.slash_menu_confirm(),
            KeyCode::Backspace => app.input_pop_char(),
            KeyCode::Char(c) => app.input_push_char(c),
            _ => {}
        }
        return;
    }

    match code {
        KeyCode::Enter => app.confirm_new_task(),
        KeyCode::Esc => app.cancel_insert_mode(),
        KeyCode::Backspace => app.input_pop_char(),
        KeyCode::Char('/') => app.open_slash_menu(),
        KeyCode::Char(c) => app.input_push_char(c),
        _ => {}
    }
}

fn handle_category_screen_key(app: &mut App, code: KeyCode) {
    match app.category_screen_mode {
        CategoryScreenMode::Browsing => match code {
            KeyCode::Esc | KeyCode::Char('q') => app.leave_category_screen(),
            KeyCode::Down | KeyCode::Char('j') => app.category_select_next(),
            KeyCode::Up | KeyCode::Char('k') => app.category_select_prev(),
            KeyCode::Char('a') => app.start_new_category(),
            KeyCode::Char('e') | KeyCode::Enter => app.start_edit_selected_category(),
            KeyCode::Char('d') => app.request_delete_category(),
            _ => {}
        },
        CategoryScreenMode::Editing => match code {
            KeyCode::Esc => app.cancel_category_form(),
            KeyCode::Enter => app.confirm_category_form(),
            KeyCode::Left => app.category_color_prev(),
            KeyCode::Right => app.category_color_next(),
            KeyCode::Backspace => app.category_name_pop(),
            KeyCode::Char(c) => app.category_name_push(c),
            _ => {}
        },
        CategoryScreenMode::ConfirmDelete => match code {
            KeyCode::Char('y') | KeyCode::Enter => app.confirm_delete_category(),
            KeyCode::Char('n') | KeyCode::Esc => app.cancel_delete_category(),
            _ => {}
        },
    }
}
