mod categories;
mod list;

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppMode, CategoryScreenMode, InputMode};

pub(crate) fn draw_ui(f: &mut Frame, app: &mut App) {
    app.layout.task_list = None;
    app.layout.category_list = None;
    app.layout.slash_menu = None;
    app.layout.slash_menu_items.clear();
    app.layout.color_cells.clear();

    match app.screen {
        AppMode::List => list::draw_list_screen(f, app),
        AppMode::CategoryEdit => categories::draw_category_edit_screen(f, app),
    }
}

pub(super) fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
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
