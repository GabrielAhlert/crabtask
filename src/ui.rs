mod categories;
mod list;

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, AppMode, CategoryScreenMode, FooterAction, FooterHint, InputMode};

type HintGroup = (&'static str, String, Color, Option<FooterAction>);

pub(crate) fn draw_ui(f: &mut Frame, app: &mut App) {
    app.layout.task_list = None;
    app.layout.category_list = None;
    app.layout.input = None;
    app.layout.slash_menu = None;
    app.layout.slash_menu_items.clear();
    app.layout.color_cells.clear();
    app.layout.footer_hints.clear();

    match app.screen {
        AppMode::List => list::draw_list_screen(f, app),
        AppMode::CategoryEdit => categories::draw_category_edit_screen(f, app),
    }
}

pub(super) fn draw_footer(f: &mut Frame, area: Rect, app: &mut App) {
    let groups: Vec<HintGroup> = match app.screen {
        AppMode::List => match app.mode {
            InputMode::Normal => {
                let inactive_label = if app.show_inactive {
                    "ocultar inativas".to_string()
                } else {
                    "mostrar inativas".to_string()
                };
                vec![
                    (" ↑/↓ ", "nav".to_string(), Color::Cyan, None),
                    (
                        " a ",
                        "add".to_string(),
                        Color::Cyan,
                        Some(FooterAction::EnterInsert),
                    ),
                    (
                        " Esp ",
                        "toggle".to_string(),
                        Color::Cyan,
                        Some(FooterAction::ToggleDone),
                    ),
                    (
                        " x ",
                        "ativa/inativa".to_string(),
                        Color::Cyan,
                        Some(FooterAction::ToggleActive),
                    ),
                    (
                        " i ",
                        inactive_label,
                        Color::Cyan,
                        Some(FooterAction::ToggleShowInactive),
                    ),
                    (
                        " d ",
                        "del".to_string(),
                        Color::Cyan,
                        Some(FooterAction::DeleteSelected),
                    ),
                    (
                        " c ",
                        "categorias".to_string(),
                        Color::Cyan,
                        Some(FooterAction::EnterCategoryEdit),
                    ),
                    (
                        " q/Esc ",
                        "sair".to_string(),
                        Color::Cyan,
                        Some(FooterAction::Quit),
                    ),
                ]
            }
            InputMode::Inserting => {
                if app.slash_menu.is_some() {
                    vec![
                        (" ↑/↓ ", "nav".to_string(), Color::Cyan, None),
                        (
                            " Enter ",
                            "escolher".to_string(),
                            Color::Green,
                            Some(FooterAction::SlashMenuConfirm),
                        ),
                        (
                            " Esc ",
                            "fechar (mantém texto)".to_string(),
                            Color::Red,
                            Some(FooterAction::SlashMenuClose),
                        ),
                        (" digite ", "filtrar".to_string(), Color::Cyan, None),
                    ]
                } else {
                    vec![
                        (
                            " Enter ",
                            "confirmar".to_string(),
                            Color::Green,
                            Some(FooterAction::ConfirmNewTask),
                        ),
                        (
                            " / ",
                            "categoria".to_string(),
                            Color::Cyan,
                            Some(FooterAction::OpenSlashMenu),
                        ),
                        (
                            " Esc ",
                            "cancelar".to_string(),
                            Color::Red,
                            Some(FooterAction::CancelInsert),
                        ),
                    ]
                }
            }
        },
        AppMode::CategoryEdit => match app.category_screen_mode {
            CategoryScreenMode::Browsing => vec![
                (" ↑/↓ ", "nav".to_string(), Color::Cyan, None),
                (
                    " a ",
                    "nova".to_string(),
                    Color::Cyan,
                    Some(FooterAction::NewCategory),
                ),
                (
                    " e/Enter ",
                    "editar".to_string(),
                    Color::Cyan,
                    Some(FooterAction::EditCategory),
                ),
                (
                    " d ",
                    "deletar".to_string(),
                    Color::Cyan,
                    Some(FooterAction::DeleteCategory),
                ),
                (
                    " Esc ",
                    "voltar".to_string(),
                    Color::Red,
                    Some(FooterAction::LeaveCategoryScreen),
                ),
            ],
            CategoryScreenMode::Editing => vec![
                (
                    " ← ",
                    "cor anterior".to_string(),
                    Color::Cyan,
                    Some(FooterAction::CategoryColorPrev),
                ),
                (
                    " → ",
                    "próxima cor".to_string(),
                    Color::Cyan,
                    Some(FooterAction::CategoryColorNext),
                ),
                (" digite ", "nome".to_string(), Color::Cyan, None),
                (
                    " Enter ",
                    "salvar".to_string(),
                    Color::Green,
                    Some(FooterAction::ConfirmCategoryForm),
                ),
                (
                    " Esc ",
                    "cancelar".to_string(),
                    Color::Red,
                    Some(FooterAction::CancelCategoryForm),
                ),
            ],
            CategoryScreenMode::ConfirmDelete => vec![
                (
                    " y/Enter ",
                    "confirmar".to_string(),
                    Color::Green,
                    Some(FooterAction::ConfirmDeleteCategory),
                ),
                (
                    " n/Esc ",
                    "cancelar".to_string(),
                    Color::Red,
                    Some(FooterAction::CancelDeleteCategory),
                ),
            ],
        },
    };

    let mut spans: Vec<Span> = Vec::new();
    let mut hints: Vec<FooterHint> = Vec::new();
    let inner_left = area.x + 1;
    let inner_top = area.y + 1;
    let mut cursor: u16 = 0;
    for (i, (key, label, color, action)) in groups.iter().enumerate() {
        let key_w = key.chars().count() as u16;
        let label_w = label.chars().count() as u16;
        let total = key_w + label_w;
        if let Some(a) = action {
            hints.push(FooterHint {
                area: Rect {
                    x: inner_left + cursor,
                    y: inner_top,
                    width: total,
                    height: 1,
                },
                action: *a,
            });
        }
        spans.push(Span::styled(
            *key,
            Style::default().fg(*color).add_modifier(ratatui::style::Modifier::BOLD),
        ));
        spans.push(Span::raw(label.clone()));
        cursor += total;
        if i + 1 < groups.len() {
            spans.push(Span::raw("  "));
            cursor += 2;
        }
    }
    app.layout.footer_hints = hints;

    let line = if let Some(msg) = &app.status {
        Line::from(Span::styled(
            msg.clone(),
            Style::default().fg(Color::Yellow).bold(),
        ))
    } else {
        Line::from(spans)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let paragraph = Paragraph::new(line).block(block);
    f.render_widget(paragraph, area);
}
