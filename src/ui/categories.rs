use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, CategoryColor, CategoryScreenMode};

pub(super) fn draw_category_edit_screen(f: &mut Frame, app: &mut App) {
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
    super::draw_footer(f, chunks[4], app);

    if app.category_screen_mode == CategoryScreenMode::ConfirmDelete {
        draw_confirm_delete_popup(f, app);
    }
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

fn draw_category_name_field(f: &mut Frame, area: Rect, app: &mut App) {
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

fn draw_category_color_picker(f: &mut Frame, area: Rect, app: &mut App) {
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

    let inner_top = area.y + 1;
    let inner_left = area.x + 1;
    let mut x_offset: u16 = 0;
    let mut spans: Vec<Span> = Vec::new();
    let mut color_cells: Vec<Rect> = Vec::with_capacity(CategoryColor::ALL.len());
    for (i, color) in CategoryColor::ALL.iter().enumerate() {
        let is_selected = editing && i == app.category_color_index;
        let swatch = if is_selected { "▶■ " } else { " ■ " };
        let swatch_cells: u16 = 3;
        let label_cells: u16 = color.label().chars().count() as u16;
        let cell_width = swatch_cells + label_cells;

        color_cells.push(Rect {
            x: inner_left + x_offset,
            y: inner_top,
            width: cell_width,
            height: 1,
        });

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

        x_offset += cell_width;
        if i + 1 < CategoryColor::ALL.len() {
            spans.push(Span::raw("  "));
            x_offset += 2;
        }
    }
    app.layout.color_cells = color_cells;

    let p = Paragraph::new(Line::from(spans))
        .block(block)
        .wrap(Wrap { trim: true });
    f.render_widget(p, area);
}

fn draw_category_list(f: &mut Frame, area: Rect, app: &mut App) {
    app.layout.category_list = Some(area);
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

    f.render_stateful_widget(list, area, &mut app.category_list_state);
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
