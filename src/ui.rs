use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode, CategoryColor, CategoryFocus, InputMode, Task};

pub(crate) fn draw_ui(f: &mut Frame, app: &App) {
    match app.screen {
        AppMode::List => draw_list_screen(f, app),
        AppMode::CategoryEdit => draw_category_edit_screen(f, app),
    }
}

fn draw_list_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // progress gauge
            Constraint::Length(3), // header
            Constraint::Min(3),    // task list + details
            Constraint::Length(3), // input box
            Constraint::Length(3), // help / status
        ])
        .split(f.area());

    let main_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[2]);

    draw_progress(f, chunks[0], app);
    draw_header(f, chunks[1], app);
    draw_task_list(f, main_panels[0], app);
    draw_details(f, main_panels[1], app);
    draw_input(f, chunks[3], app);
    draw_footer(f, chunks[4], app);
}

fn draw_category_edit_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Length(3), // name input
            Constraint::Length(3), // color picker
            Constraint::Min(3),    // existing categories list
            Constraint::Length(3), // footer
        ])
        .split(f.area());

    draw_category_header(f, chunks[0]);
    draw_category_name_field(f, chunks[1], app);
    draw_category_color_picker(f, chunks[2], app);
    draw_category_list(f, chunks[3], app);
    draw_footer(f, chunks[4], app);
}

fn format_local(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&Local).format("%d/%m/%Y %H:%M:%S").to_string()
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
        Span::raw("  •  categorias: "),
        Span::styled(
            app.categories.len().to_string(),
            Style::default().fg(Color::Cyan).bold(),
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
            let (marker, marker_style) = if task.done {
                ("[X] ", Style::default().fg(Color::Green).bold())
            } else {
                ("[ ] ", Style::default().fg(Color::Yellow).bold())
            };

            // Title color follows the first tag's category color when present.
            let tag_color = task
                .tags
                .first()
                .and_then(|t| app.category_color(t))
                .map(|c| c.to_color());

            let title_style = match (task.done, tag_color) {
                (true, Some(c)) => Style::default()
                    .fg(c)
                    .add_modifier(Modifier::CROSSED_OUT),
                (true, None) => Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::CROSSED_OUT),
                (false, Some(c)) => Style::default().fg(c).bold(),
                (false, None) => Style::default().fg(Color::White),
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

fn draw_details(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Detalhes ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ))
        .border_style(Style::default().fg(Color::DarkGray));

    let selected: Option<&Task> = app
        .list_state
        .selected()
        .and_then(|i| app.tasks.get(i));

    let Some(task) = selected else {
        let empty = Paragraph::new("\n  Nenhuma tarefa selecionada.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(empty, area);
        return;
    };

    let status_line = if task.done {
        Line::from(vec![
            Span::raw("  Status: "),
            Span::styled("concluída", Style::default().fg(Color::Green).bold()),
        ])
    } else {
        Line::from(vec![
            Span::raw("  Status: "),
            Span::styled("pendente", Style::default().fg(Color::Yellow).bold()),
        ])
    };

    let title_line = Line::from(vec![
        Span::raw("  Título: "),
        Span::styled(
            task.title.clone(),
            Style::default().fg(Color::White).bold(),
        ),
    ]);

    let created_line = Line::from(vec![
        Span::raw("  Criada em:    "),
        Span::styled(
            format_local(task.created_at),
            Style::default().fg(Color::Cyan),
        ),
    ]);

    let completed_line = match task.completed_at {
        Some(ts) => Line::from(vec![
            Span::raw("  Concluída em: "),
            Span::styled(format_local(ts), Style::default().fg(Color::Green)),
        ]),
        None => Line::from(vec![
            Span::raw("  Concluída em: "),
            Span::styled("—", Style::default().fg(Color::DarkGray)),
        ]),
    };

    let tags_line = if task.tags.is_empty() {
        Line::from(vec![
            Span::raw("  Tags: "),
            Span::styled("—", Style::default().fg(Color::DarkGray)),
        ])
    } else {
        let mut spans: Vec<Span> = vec![Span::raw("  Tags: ")];
        for (i, tag) in task.tags.iter().enumerate() {
            let color = app
                .category_color(tag)
                .map(|c| c.to_color())
                .unwrap_or(Color::DarkGray);
            spans.push(Span::styled(
                format!("#{}", tag),
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            ));
            if i + 1 < task.tags.len() {
                spans.push(Span::raw("  "));
            }
        }
        Line::from(spans)
    };

    let lines = vec![
        Line::from(""),
        title_line,
        Line::from(""),
        status_line,
        Line::from(""),
        created_line,
        completed_line,
        Line::from(""),
        tags_line,
    ];

    let paragraph = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
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
    let help = match app.screen {
        AppMode::List => match app.mode {
            InputMode::Normal => Line::from(vec![
                Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("nav  "),
                Span::styled(" a ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("add  "),
                Span::styled(" Esp ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("toggle  "),
                Span::styled(" d ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("del  "),
                Span::styled(" 1-9 ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("tag  "),
                Span::styled(" c ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("categorias  "),
                Span::styled(" q/Esc ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("sair"),
            ]),
            InputMode::Inserting => Line::from(vec![
                Span::styled(" Enter ", Style::default().fg(Color::Green).bold()),
                Span::raw("confirmar  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
                Span::raw("cancelar  "),
                Span::styled(" Backspace ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("apagar"),
            ]),
        },
        AppMode::CategoryEdit => Line::from(vec![
            Span::styled(" Tab ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("alternar foco  "),
            Span::styled(" ←/→ ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("cor  "),
            Span::styled(" Enter ", Style::default().fg(Color::Green).bold()),
            Span::raw("salvar  "),
            Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
            Span::raw("voltar"),
        ]),
    };

    let line = if let Some(msg) = &app.status {
        Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(Color::Yellow).bold(),
        ))
    } else {
        help
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}

fn draw_category_header(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            "  Editor de Categorias 🦀 ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ),
        Span::styled(
            "— Tab alterna foco · Enter salva · Esc volta",
            Style::default().fg(Color::Gray),
        ),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 140, 60)));
    let p = Paragraph::new(title).block(block);
    f.render_widget(p, area);
}

fn draw_category_name_field(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.category_focus == CategoryFocus::Name;
    let border_color = if focused {
        Color::Rgb(255, 140, 60)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Nome da categoria ",
            Style::default().fg(border_color).bold(),
        ))
        .border_style(Style::default().fg(border_color));

    let content = app.category_name_buffer.clone();
    let style = if focused {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::Gray)
    };
    let p = Paragraph::new(content.clone()).style(style).block(block);
    f.render_widget(p, area);

    if focused {
        let x = area.x + 1 + content.chars().count() as u16;
        let y = area.y + 1;
        let max_x = area.x + area.width.saturating_sub(2);
        f.set_cursor_position((x.min(max_x), y));
    }
}

fn draw_category_color_picker(f: &mut Frame, area: Rect, app: &App) {
    let focused = app.category_focus == CategoryFocus::Color;
    let border_color = if focused {
        Color::Rgb(255, 140, 60)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Cor (←/→) ",
            Style::default().fg(border_color).bold(),
        ))
        .border_style(Style::default().fg(border_color));

    let mut spans: Vec<Span> = Vec::new();
    for (i, color) in CategoryColor::ALL.iter().enumerate() {
        let is_selected = i == app.category_color_index;
        let swatch = if is_selected { "▶■ " } else { " ■ " };
        let swatch_style = if is_selected {
            Style::default().fg(color.to_color()).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color.to_color())
        };
        spans.push(Span::styled(swatch, swatch_style));
        let label_style = if is_selected {
            Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(color.label(), label_style));
        if i + 1 < CategoryColor::ALL.len() {
            spans.push(Span::raw("  "));
        }
    }

    let p = Paragraph::new(Line::from(spans))
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_category_list(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Categorias existentes ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ))
        .border_style(Style::default().fg(Color::DarkGray));

    if app.categories.is_empty() {
        let p = Paragraph::new("\n  Nenhuma categoria criada ainda.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = app
        .categories
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let line = Line::from(vec![
                Span::styled(
                    format!("  {}.  ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    "■ ",
                    Style::default().fg(c.color.to_color()).add_modifier(Modifier::BOLD),
                ),
                Span::styled(c.name.clone(), Style::default().fg(Color::White)),
                Span::styled(
                    format!("  ({})", c.color.label()),
                    Style::default().fg(Color::DarkGray),
                ),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}
