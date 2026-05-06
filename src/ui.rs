use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode};

pub(crate) fn draw_ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // progress gauge
            Constraint::Length(3), // header
            Constraint::Min(3),    // task list
            Constraint::Length(3), // input box
            Constraint::Length(3), // help / status
        ])
        .split(f.area());

    draw_progress(f, chunks[0], app);
    draw_header(f, chunks[1], app);
    draw_task_list(f, chunks[2], app);
    draw_input(f, chunks[3], app);
    draw_footer(f, chunks[4], app);
}

fn draw_progress(f: &mut Frame, area: Rect, app: &App) {
    let total = app.tasks.len();
    let done = app.done_count();
    let ratio = app.progress_ratio();
    let percent = (ratio * 100.0).round() as u16;

    let gauge_color = if total == 0 {
        Color::DarkGray
    } else if percent >= 100 {
        Color::Green
    } else if percent >= 67 {
        Color::LightGreen
    } else if percent >= 34 {
        Color::Yellow
    } else if percent >= 1 {
        Color::Red
    } else {
        Color::DarkGray
    };

    let label = if total == 0 {
        "sem tarefas".to_string()
    } else {
        format!("{}/{}  •  {}%", done, total, percent)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Progresso ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ))
        .border_style(Style::default().fg(Color::Rgb(255, 140, 60)));

    let gauge = Gauge::default()
        .block(block)
        .gauge_style(
            Style::default()
                .fg(gauge_color)
                .bg(Color::Rgb(30, 30, 30))
                .add_modifier(Modifier::BOLD),
        )
        .ratio(ratio)
        .label(Span::styled(
            label,
            Style::default().fg(Color::White).bold(),
        ));

    f.render_widget(gauge, area);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let title = Line::from(vec![
        Span::styled("  CrabTask 🦀 ", Style::default().fg(Color::Rgb(255, 140, 60)).bold()),
        Span::styled("— TUI To-Do em Rust", Style::default().fg(Color::Gray)),
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
        let x = area.x + 1 + content.chars().count() as u16;
        let y = area.y + 1;
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
