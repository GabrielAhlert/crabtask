use std::{
    fs,
    io::{self, Stdout},
    panic,
    path::PathBuf,
    time::Duration,
};

use chrono::{DateTime, Utc};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use serde::{Deserialize, Serialize};

const STORAGE_FILE: &str = "tasks.json";

// ---------------------------------------------------------------------------
// Domain model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Task {
    title: String,
    done: bool,
    #[serde(default = "Utc::now")]
    created_at: DateTime<Utc>,
    #[serde(default)]
    completed_at: Option<DateTime<Utc>>,
}

impl Task {
    fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            done: false,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

// ---------------------------------------------------------------------------
// App state
// ---------------------------------------------------------------------------

#[derive(Debug, PartialEq, Eq)]
enum InputMode {
    Normal,
    Inserting,
}

struct App {
    tasks: Vec<Task>,
    list_state: ListState,
    mode: InputMode,
    input_buffer: String,
    status: Option<String>,
    should_quit: bool,
}

impl App {
    fn new(tasks: Vec<Task>) -> Self {
        let mut list_state = ListState::default();
        if !tasks.is_empty() {
            list_state.select(Some(0));
        }
        Self {
            tasks,
            list_state,
            mode: InputMode::Normal,
            input_buffer: String::new(),
            status: None,
            should_quit: false,
        }
    }

    fn select_next(&mut self) {
        if self.tasks.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.tasks.len() => i + 1,
            Some(_) => 0, // wrap around
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) => self.tasks.len() - 1, // wrap around
            Some(i) => i - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    fn toggle_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(task) = self.tasks.get_mut(i) {
                task.done = !task.done;
                task.completed_at = if task.done { Some(Utc::now()) } else { None };
            }
        }
    }

    fn delete_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if i < self.tasks.len() {
                self.tasks.remove(i);
                if self.tasks.is_empty() {
                    self.list_state.select(None);
                } else if i >= self.tasks.len() {
                    self.list_state.select(Some(self.tasks.len() - 1));
                }
            }
        }
    }

    fn enter_insert_mode(&mut self) {
        self.mode = InputMode::Inserting;
        self.input_buffer.clear();
        self.status = None;
    }

    fn cancel_insert_mode(&mut self) {
        self.mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    fn confirm_new_task(&mut self) {
        let title = self.input_buffer.trim();
        if !title.is_empty() {
            self.tasks.push(Task::new(title));
            // Select the newly created task.
            self.list_state.select(Some(self.tasks.len() - 1));
        }
        self.input_buffer.clear();
        self.mode = InputMode::Normal;
    }

    fn pending_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.done).count()
    }

    fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }
}

// ---------------------------------------------------------------------------
// Persistence
// ---------------------------------------------------------------------------

fn storage_path() -> PathBuf {
    PathBuf::from(STORAGE_FILE)
}

fn load_tasks() -> Vec<Task> {
    let path = storage_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) if content.trim().is_empty() => Vec::new(),
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

fn save_tasks(tasks: &[Task]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(storage_path(), json)
}

// ---------------------------------------------------------------------------
// Terminal lifecycle
// ---------------------------------------------------------------------------

type Tui = Terminal<CrosstermBackend<Stdout>>;

fn init_terminal() -> io::Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

fn restore_terminal(terminal: &mut Tui) -> io::Result<()> {
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
fn force_restore_terminal() {
    let _ = disable_raw_mode();
    let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
}

// ---------------------------------------------------------------------------
// Event loop
// ---------------------------------------------------------------------------

fn run_app(terminal: &mut Tui, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|f| draw_ui(f, app))?;

        // Poll keeps the UI responsive even without input.
        if event::poll(Duration::from_millis(200))? {
            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }
                match app.mode {
                    InputMode::Normal => handle_normal_key(app, key.code),
                    InputMode::Inserting => handle_insert_key(app, key.code),
                }
            }
        }
    }
    Ok(())
}

fn handle_normal_key(app: &mut App, code: KeyCode) {
    match code {
        KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
        KeyCode::Char('a') => app.enter_insert_mode(),
        KeyCode::Char('d') => app.delete_selected(),
        KeyCode::Char(' ') => app.toggle_selected(),
        KeyCode::Down | KeyCode::Char('j') => app.select_next(),
        KeyCode::Up | KeyCode::Char('k') => app.select_previous(),
        _ => {}
    }
}

fn handle_insert_key(app: &mut App, code: KeyCode) {
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

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(3),    // task list
            Constraint::Length(3), // input box
            Constraint::Length(3), // help / status
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_task_list(f, chunks[1], app);
    draw_input(f, chunks[2], app);
    draw_footer(f, chunks[3], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled("  CrabTask 🦀 ", Style::default().fg(Color::Rgb(255, 140, 60)).bold()),
        Span::styled(
            "— TUI To-Do em Rust",
            Style::default().fg(Color::Gray),
        ),
    ]);

    let stats = Line::from(vec![
        Span::raw("  total: "),
        Span::styled(
            app.tasks.len().to_string(),
            Style::default().fg(Color::White).bold(),
        ),
        Span::raw("  •  pendentes: "),
        Span::styled(
            app.pending_count().to_string(),
            Style::default().fg(Color::Yellow).bold(),
        ),
        Span::raw("  •  concluídas: "),
        Span::styled(
            app.done_count().to_string(),
            Style::default().fg(Color::Green).bold(),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 140, 60)));

    let paragraph = Paragraph::new(vec![title, stats]).block(block);
    f.render_widget(paragraph, area);
}

fn draw_task_list(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Tarefas ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ));

    if app.tasks.is_empty() {
        let empty = Paragraph::new(
            "\n  Nenhuma tarefa ainda. Pressione 'a' para adicionar uma.",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(block)
        .wrap(Wrap { trim: true });
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .map(|task| {
            let (marker, marker_style, title_style) = if task.done {
                (
                    "[X] ",
                    Style::default().fg(Color::Green).bold(),
                    Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT),
                )
            } else {
                (
                    "[ ] ",
                    Style::default().fg(Color::Yellow).bold(),
                    Style::default().fg(Color::White),
                )
            };

            let line = Line::from(vec![
                Span::styled(marker, marker_style),
                Span::styled(task.title.clone(), title_style),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol(" ▶ ")
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = app.list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_input(f: &mut Frame, area: Rect, app: &App) {
    let (title, content, style) = match app.mode {
        InputMode::Normal => (
            " Nova tarefa (pressione 'a') ",
            String::new(),
            Style::default().fg(Color::DarkGray),
        ),
        InputMode::Inserting => (
            " Nova tarefa — Enter confirma, Esc cancela ",
            app.input_buffer.clone(),
            Style::default().fg(Color::White),
        ),
    };

    let border_color = if app.mode == InputMode::Inserting {
        Color::Rgb(255, 140, 60)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(title, Style::default().fg(border_color).bold()))
        .border_style(Style::default().fg(border_color));

    let paragraph = Paragraph::new(content.clone()).style(style).block(block);
    f.render_widget(paragraph, area);

    if app.mode == InputMode::Inserting {
        // Place cursor right after the typed text.
        let x = area.x + 1 + content.chars().count() as u16;
        let y = area.y + 1;
        // Clamp to area to avoid going outside.
        let max_x = area.x + area.width.saturating_sub(2);
        f.set_cursor_position((x.min(max_x), y));
    }
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help = match app.mode {
        InputMode::Normal => Line::from(vec![
            Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("navegar  "),
            Span::styled(" a ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("adicionar  "),
            Span::styled(" Espaço ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("alternar  "),
            Span::styled(" d ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("deletar  "),
            Span::styled(" q/Esc ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("salvar e sair"),
        ]),
        InputMode::Inserting => Line::from(vec![
            Span::styled(" Enter ", Style::default().fg(Color::Green).bold()),
            Span::raw("confirmar  "),
            Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
            Span::raw("cancelar  "),
            Span::styled(" Backspace ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("apagar"),
        ]),
    };

    let lines = if let Some(msg) = &app.status {
        vec![help, Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Yellow)))]
    } else {
        vec![help]
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(lines).block(block);
    f.render_widget(paragraph, area);
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> io::Result<()> {
    // Make sure the terminal is restored even if we panic in the middle of drawing.
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        force_restore_terminal();
        original_hook(info);
    }));

    let tasks = load_tasks();
    let mut app = App::new(tasks);

    let mut terminal = init_terminal()?;
    let run_result = run_app(&mut terminal, &mut app);
    let restore_result = restore_terminal(&mut terminal);

    // Always try to persist whatever state we have, even after a partial error.
    let save_result = save_tasks(&app.tasks);

    run_result?;
    restore_result?;
    save_result?;

    println!(
        "CrabTask: {} tarefa(s) salva(s) em {}",
        app.tasks.len(),
        STORAGE_FILE
    );
    Ok(())
}
