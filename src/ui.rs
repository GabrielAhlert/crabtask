use chrono::Local;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, AppMode, CategoryColor, CategoryScreenMode, InputMode, Task};

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
    draw_footer(f, chunks[3], app);

    if app.slash_menu.is_some() {
        draw_slash_menu(f, chunks[2], app);
    }
}

fn draw_category_edit_screen(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // header
            Constraint::Min(5),    // categories list
            Constraint::Length(3), // name field
            Constraint::Length(3), // color picker
            Constraint::Length(3), // footer
        ])
        .split(f.area());

    draw_category_header(f, chunks[0]);
    draw_category_list(f, chunks[1], app);
    draw_category_name_field(f, chunks[2], app);
    draw_category_color_picker(f, chunks[3], app);
    draw_footer(f, chunks[4], app);

    if app.category_screen_mode == CategoryScreenMode::ConfirmDelete {
        draw_confirm_delete_popup(f, app);
    }
}

fn format_local(ts: chrono::DateTime<chrono::Utc>) -> String {
    ts.with_timezone(&Local)
        .format("%d/%m/%Y %H:%M:%S")
        .to_string()
}

fn draw_progress(f: &mut Frame, area: Rect, app: &App) {
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

fn draw_task_list(f: &mut Frame, area: Rect, app: &App) {
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

fn draw_input(f: &mut Frame, area: Rect, app: &App) {
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

fn draw_slash_menu(f: &mut Frame, input_area: Rect, app: &App) {
    let Some(menu) = &app.slash_menu else {
        return;
    };
    let filtered = app.slash_filtered_indices();

    let max_visible = 6usize;
    let visible_count = filtered.len().clamp(1, max_visible);
    let height = visible_count as u16 + 2;
    let width: u16 = 32;

    let chars_before_slash = app.input_buffer[..menu.slash_pos].chars().count() as u16;
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

    let title_text = if app.slash_query().is_empty() {
        " Categorias ".to_string()
    } else {
        format!(" Categorias · /{} ", app.slash_query())
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
    state.select(Some(menu.selected.min(filtered.len().saturating_sub(1))));
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let help = match app.screen {
        AppMode::List => match app.mode {
            InputMode::Normal => {
                let inactive_label = if app.show_inactive {
                    "ocultar inativas"
                } else {
                    "mostrar inativas"
                };
                Line::from(vec![
                    Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("nav  "),
                    Span::styled(" a ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("add  "),
                    Span::styled(" Esp ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("toggle  "),
                    Span::styled(" x ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("ativa/inativa  "),
                    Span::styled(" i ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw(inactive_label),
                    Span::raw("  "),
                    Span::styled(" d ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("del  "),
                    Span::styled(" c ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("categorias  "),
                    Span::styled(" q/Esc ", Style::default().fg(Color::Cyan).bold()),
                    Span::raw("sair"),
                ])
            }
            InputMode::Inserting => {
                if app.slash_menu.is_some() {
                    Line::from(vec![
                        Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("nav  "),
                        Span::styled(" Enter ", Style::default().fg(Color::Green).bold()),
                        Span::raw("escolher  "),
                        Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
                        Span::raw("fechar (mantém texto)  "),
                        Span::styled(" digite ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("filtrar"),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(" Enter ", Style::default().fg(Color::Green).bold()),
                        Span::raw("confirmar  "),
                        Span::styled(" / ", Style::default().fg(Color::Cyan).bold()),
                        Span::raw("categoria  "),
                        Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
                        Span::raw("cancelar"),
                    ])
                }
            }
        },
        AppMode::CategoryEdit => match app.category_screen_mode {
            CategoryScreenMode::Browsing => Line::from(vec![
                Span::styled(" ↑/↓ ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("nav  "),
                Span::styled(" a ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("nova  "),
                Span::styled(" e/Enter ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("editar  "),
                Span::styled(" d ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("deletar  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
                Span::raw("voltar"),
            ]),
            CategoryScreenMode::Editing => Line::from(vec![
                Span::styled(" ←/→ ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("cor  "),
                Span::styled(" digite ", Style::default().fg(Color::Cyan).bold()),
                Span::raw("nome  "),
                Span::styled(" Enter ", Style::default().fg(Color::Green).bold()),
                Span::raw("salvar  "),
                Span::styled(" Esc ", Style::default().fg(Color::Red).bold()),
                Span::raw("cancelar"),
            ]),
            CategoryScreenMode::ConfirmDelete => Line::from(vec![
                Span::styled(" y/Enter ", Style::default().fg(Color::Green).bold()),
                Span::raw("confirmar  "),
                Span::styled(" n/Esc ", Style::default().fg(Color::Red).bold()),
                Span::raw("cancelar"),
            ]),
        },
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
            "— navegue na lista e use n/e/d",
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
    let editing = app.category_screen_mode == CategoryScreenMode::Editing;
    let title_str = if editing {
        match app.editing_category_index {
            Some(_) => " Editando categoria — digite o nome ",
            None => " Nova categoria — digite o nome ",
        }
    } else {
        " Nome (sem edição ativa) "
    };
    let border_color = if editing {
        Color::Rgb(255, 140, 60)
    } else {
        Color::DarkGray
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            title_str,
            Style::default().fg(border_color).bold(),
        ))
        .border_style(Style::default().fg(border_color));

    let content = if editing {
        app.category_name_buffer.clone()
    } else {
        String::new()
    };
    let style = if editing {
        Style::default().fg(Color::White)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let p = Paragraph::new(content.clone()).style(style).block(block);
    f.render_widget(p, area);

    if editing {
        let x = area.x + 1 + content.chars().count() as u16;
        let y = area.y + 1;
        let max_x = area.x + area.width.saturating_sub(2);
        f.set_cursor_position((x.min(max_x), y));
    }
}

fn draw_category_color_picker(f: &mut Frame, area: Rect, app: &App) {
    let editing = app.category_screen_mode == CategoryScreenMode::Editing;
    let border_color = if editing {
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
        let is_selected = editing && i == app.category_color_index;
        let swatch = if is_selected { "▶■ " } else { " ■ " };
        let swatch_style = if is_selected {
            Style::default()
                .fg(color.to_color())
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(color.to_color())
        };
        spans.push(Span::styled(swatch, swatch_style));
        let label_style = if is_selected {
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD)
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
            " Categorias ",
            Style::default().fg(Color::Rgb(255, 140, 60)).bold(),
        ))
        .border_style(Style::default().fg(Color::DarkGray));

    if app.categories.is_empty() {
        let p = Paragraph::new("\n  Nenhuma categoria criada. Pressione 'a' para criar.")
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
            let usage = app.category_usage_count(&c.name);
            let line = Line::from(vec![
                Span::styled(
                    format!("  {}.  ", i + 1),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    "■ ",
                    Style::default()
                        .fg(c.color.to_color())
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(c.name.clone(), Style::default().fg(Color::White).bold()),
                Span::styled(
                    format!("  ({})", c.color.label()),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::raw("  · "),
                Span::styled(
                    format!("{} task(s)", usage),
                    Style::default().fg(Color::Cyan),
                ),
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

    let mut state = app.category_list_state.clone();
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_confirm_delete_popup(f: &mut Frame, app: &App) {
    let frame_area = f.area();
    let width: u16 = 60;
    let height: u16 = 7;
    let x = frame_area.x + frame_area.width.saturating_sub(width) / 2;
    let y = frame_area.y + frame_area.height.saturating_sub(height) / 2;
    let area = Rect {
        x,
        y,
        width: width.min(frame_area.width),
        height: height.min(frame_area.height),
    };

    let cat_name = app
        .category_list_state
        .selected()
        .and_then(|i| app.categories.get(i))
        .map(|c| c.name.clone())
        .unwrap_or_default();
    let usage = app.category_usage_count(&cat_name);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            " Confirmar exclusão ",
            Style::default().fg(Color::Red).bold(),
        ))
        .border_style(Style::default().fg(Color::Red));

    let lines = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("  Excluir categoria "),
            Span::styled(
                format!("'{}'", cat_name),
                Style::default().fg(Color::White).bold(),
            ),
            Span::raw("?"),
        ]),
        Line::from(vec![
            Span::raw("  Será removida de "),
            Span::styled(
                format!("{}", usage),
                Style::default().fg(Color::Yellow).bold(),
            ),
            Span::raw(" task(s)."),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  y/Enter ", Style::default().fg(Color::Green).bold()),
            Span::raw("confirmar    "),
            Span::styled(" n/Esc ", Style::default().fg(Color::Cyan).bold()),
            Span::raw("cancelar"),
        ]),
    ];

    let p = Paragraph::new(lines).block(block);
    f.render_widget(Clear, area);
    f.render_widget(p, area);
}
