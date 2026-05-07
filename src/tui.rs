use std::{
    io::{self, Stdout},
    time::Duration,
};

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};

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
            match event::read()? {
                Event::Key(key) => {
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
                Event::Mouse(m) => match app.screen {
                    AppMode::List => handle_list_mouse(app, m),
                    AppMode::CategoryEdit => handle_category_mouse(app, m),
                },
                _ => {}
            }
        }
    }
    Ok(())
}

fn rect_contains(r: Rect, col: u16, row: u16) -> bool {
    col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
}

fn handle_list_mouse(app: &mut App, m: MouseEvent) {
    match m.kind {
        MouseEventKind::ScrollUp => {
            if let Some(r) = app.layout.task_list {
                if rect_contains(r, m.column, m.row) {
                    app.select_previous();
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if let Some(r) = app.layout.task_list {
                if rect_contains(r, m.column, m.row) {
                    app.select_next();
                }
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(menu_rect) = app.layout.slash_menu {
                if rect_contains(menu_rect, m.column, m.row) {
                    handle_slash_menu_click(app, m);
                    return;
                }
            }
            if let Some(r) = app.layout.task_list {
                if rect_contains(r, m.column, m.row) {
                    handle_task_list_click(app, r, m);
                }
            }
        }
        _ => {}
    }
}

fn handle_task_list_click(app: &mut App, area: Rect, m: MouseEvent) {
    let inner_top = area.y + 1;
    let inner_left = area.x + 1;
    let inner_height = area.height.saturating_sub(2);
    if m.row < inner_top || m.row >= inner_top + inner_height {
        return;
    }
    let row_offset = (m.row - inner_top) as usize;
    let target = app.list_state.offset() + row_offset;
    if target >= app.visible_indices().len() {
        return;
    }
    app.list_state.select(Some(target));
    // Marker zone (after the " ▶ " highlight prefix): cols [inner+3 .. inner+7) = "[X] "
    if m.column >= inner_left + 3 && m.column < inner_left + 7 {
        app.toggle_selected();
    }
}

fn handle_slash_menu_click(app: &mut App, m: MouseEvent) {
    let items: Vec<Rect> = app.layout.slash_menu_items.clone();
    for (i, r) in items.iter().enumerate() {
        if rect_contains(*r, m.column, m.row) {
            if let Some(menu) = app.slash_menu.as_mut() {
                menu.selected = i;
            }
            app.slash_menu_confirm();
            return;
        }
    }
}

fn handle_category_mouse(app: &mut App, m: MouseEvent) {
    if app.category_screen_mode == CategoryScreenMode::ConfirmDelete {
        return;
    }
    match m.kind {
        MouseEventKind::ScrollUp => {
            if let Some(r) = app.layout.category_list {
                if rect_contains(r, m.column, m.row) {
                    app.category_select_prev();
                }
            }
        }
        MouseEventKind::ScrollDown => {
            if let Some(r) = app.layout.category_list {
                if rect_contains(r, m.column, m.row) {
                    app.category_select_next();
                }
            }
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(r) = app.layout.category_list {
                if rect_contains(r, m.column, m.row) {
                    handle_category_list_click(app, r, m);
                    return;
                }
            }
            if app.category_screen_mode == CategoryScreenMode::Editing {
                let cells: Vec<Rect> = app.layout.color_cells.clone();
                for (i, r) in cells.iter().enumerate() {
                    if rect_contains(*r, m.column, m.row) {
                        app.category_color_index = i;
                        return;
                    }
                }
            }
        }
        _ => {}
    }
}

fn handle_category_list_click(app: &mut App, area: Rect, m: MouseEvent) {
    let inner_top = area.y + 1;
    let inner_height = area.height.saturating_sub(2);
    if m.row < inner_top || m.row >= inner_top + inner_height {
        return;
    }
    let row_offset = (m.row - inner_top) as usize;
    let target = app.category_list_state.offset() + row_offset;
    if target >= app.categories.len() {
        return;
    }
    app.category_list_state.select(Some(target));
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
