use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, InputMode, Task};

pub(super) fn draw_list_screen(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header (title + progress)
            Constraint::Min(3),    // task list + details
            Constraint::Length(3), // input box
            Constraint::Length(3), // help / status
        ])
        .split(f.area());

    let header_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(45), Constraint::Percentage(55)])
        .split(chunks[0]);

    let main_panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(chunks[1]);

    draw_header(f, header_panels[0]);
    draw_progress(f, header_panels[1], app);
    draw_task_list(f, main_panels[0], app);
    draw_details(f, main_panels[1], app);
    draw_input(f, chunks[2], app);
    super::draw_footer(f, chunks[3], app);

    if app.slash_menu.is_some() {
        draw_slash_menu(f, chunks[2], app);
    }
}

fn format_local(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&Local)
        .format("%d/%m/%Y %H:%M:%S")
        .to_string()
}

fn draw_header(f: &mut Frame, area: Rect) {
    let title = Line::from(vec![
        Span::styled(
            "  CrabTask 🦀 ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ),
        Span::styled("— TUI To-Do em Rust", Style::default().fg(Color::Gray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 140, 60)));

    let paragraph = Paragraph::new(title).block(block);
    f.render_widget(paragraph, area);
}

fn draw_progress(f: &mut Frame, area: Rect, app: &mut App) {
    let active = app.active_total();
    let done = app.done_count();
    let ratio = app.progress_ratio();
    let percent = (ratio * 100.0).round() as u16;

    let gauge_color = if active == 0 {
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

    let label = if active == 0 {
        "sem tarefas".to_string()
    } else {
        format!("{}/{}  •  {}%", done, active, percent)
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

fn draw_task_list(f: &mut Frame, area: Rect, app: &mut App) {
    app.layout.task_list = Some(area);
    let title_text = if app.show_inactive {
        " Tarefas (inc. inativas) ".to_string()
    } else {
        " Tarefas ".to_string()
    };
    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        title_text,
        Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
    ));

    if app.tasks.is_empty() {
        let empty = Paragraph::new("\n  Nenhuma tarefa ainda. Pressione 'a' para adicionar uma.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(empty, area);
        return;
    }

    let visible = app.visible_indices();
    if visible.is_empty() {
        let empty = Paragraph::new(
            "\n  Todas as tarefas estão inativas. Pressione 'i' para mostrá-las.",
        )
        .style(Style::default().fg(Color::DarkGray))
        .block(block)
        .wrap(Wrap { trim: true });
        f.render_widget(empty, area);
        return;
    }

    let items: Vec<ListItem> = visible
        .iter()
        .map(|&idx| {
            let task = &app.tasks[idx];
            let (marker, marker_style) = if !task.active {
                ("[~] ", Style::default().fg(Color::DarkGray).bold())
            } else if task.done {
                ("[X] ", Style::default().fg(Color::Green).bold())
            } else {
                ("[ ] ", Style::default().fg(Color::Yellow).bold())
            };

            let tag_color = task
                .tags
                .first()
                .and_then(|t| app.category_color(t))
                .map(|c| c.to_color());

            let title_style = if !task.active {
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM)
            } else {
                match (task.done, tag_color) {
                    (true, Some(c)) => Style::default().fg(c).add_modifier(Modifier::CROSSED_OUT),
                    (true, None) => Style::default()
                        .fg(Color::DarkGray)
                        .add_modifier(Modifier::CROSSED_OUT),
                    (false, Some(c)) => Style::default().fg(c).bold(),
                    (false, None) => Style::default().fg(Color::White),
                }
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

    f.render_stateful_widget(list, area, &mut app.list_state);
}

fn draw_details(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Detalhes ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ))
        .border_style(Style::default().fg(Color::DarkGray));

    let selected: Option<&Task> = app.selected_task_index().and_then(|i| app.tasks.get(i));

    let Some(task) = selected else {
        let empty = Paragraph::new("\n  Nenhuma tarefa selecionada.")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(empty, area);
        return;
    };

    let status_line = if !task.active {
        Line::from(vec![
            Span::raw("  Status: "),
            Span::styled("inativa", Style::default().fg(Color::DarkGray).bold()),
        ])
    } else if task.done {
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
        Span::styled(task.title.clone(), Style::default().fg(Color::White).bold()),
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

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false });
    f.render_widget(paragraph, area);
}

fn draw_input(f: &mut Frame, area: Rect, app: &mut App) {
    app.layout.input = Some(area);
    let (title_line, content, style, border_color) = match app.mode {
        InputMode::Normal => (
            Line::from(" Nova tarefa (pressione 'a') "),
            String::new(),
            Style::default().fg(Color::DarkGray),
            Color::DarkGray,
        ),
        InputMode::Inserting => {
            let mut spans = vec![Span::raw(" Nova tarefa")];
            if !app.pending_tags.is_empty() {
                spans.push(Span::raw(" · "));
                for (i, tag) in app.pending_tags.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::raw(" "));
                    }
                    let color = app
                        .category_color(tag)
                        .map(|c| c.to_color())
                        .unwrap_or(Color::DarkGray);
                    spans.push(Span::styled(
                        format!("#{}", tag),
                        Style::default().fg(color).add_modifier(Modifier::BOLD),
                    ));
                }
            }
            spans.push(Span::raw(" · Enter confirma · / categoria "));
            (
                Line::from(spans),
                app.input_buffer.clone(),
                Style::default().fg(Color::White),
                Color::Rgb(255, 140, 60),
            )
        }
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title_line)
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

fn draw_slash_menu(f: &mut Frame, input_area: Rect, app: &mut App) {
    let Some((slash_pos, menu_selected)) =
        app.slash_menu.as_ref().map(|m| (m.slash_pos, m.selected))
    else {
        return;
    };
    let filtered = app.slash_filtered_indices();
    let query = app.slash_query().to_string();

    let max_visible = 6usize;
    let visible_count = filtered.len().clamp(1, max_visible);
    let height = visible_count as u16 + 2;
    let width: u16 = 32;

    let chars_before_slash = app.input_buffer[..slash_pos].chars().count() as u16;
    let frame_area = f.area();
    let mut x = input_area.x + 1 + chars_before_slash;
    if x + width > frame_area.x + frame_area.width {
        x = (frame_area.x + frame_area.width).saturating_sub(width);
    }
    let y = input_area.y.saturating_sub(height);

    let area = Rect {
        x,
        y,
        width,
        height,
    };

    app.layout.slash_menu = Some(area);
    app.layout.slash_menu_items.clear();
    if !filtered.is_empty() {
        let inner_top = area.y + 1;
        let inner_left = area.x + 1;
        let inner_width = area.width.saturating_sub(2);
        for i in 0..filtered.len().min(max_visible) {
            app.layout.slash_menu_items.push(Rect {
                x: inner_left,
                y: inner_top + i as u16,
                width: inner_width,
                height: 1,
            });
        }
    }

    let title_text = if query.is_empty() {
        " Categorias ".to_string()
    } else {
        format!(" Categorias · /{} ", query)
    };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 140, 60)))
        .title(Span::styled(
            title_text,
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ));

    f.render_widget(Clear, area);

    if filtered.is_empty() {
        let p = Paragraph::new("  sem matches (Esc para manter texto)")
            .style(Style::default().fg(Color::DarkGray))
            .block(block)
            .wrap(Wrap { trim: true });
        f.render_widget(p, area);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|&i| {
            let cat = &app.categories[i];
            let line = Line::from(vec![
                Span::styled(
                    "■ ",
                    Style::default()
                        .fg(cat.color.to_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(cat.name.clone(), Style::default().fg(Color::White)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol("▶ ")
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD),
        );

    let mut state = ListState::default();
    state.select(Some(menu_selected.min(filtered.len().saturating_sub(1))));
    f.render_stateful_widget(list, area, &mut state);
}
