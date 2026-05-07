use chrono::{DateTime, Utc};
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum CategoryColor {
    Red,
    Yellow,
    Green,
    Cyan,
    Blue,
    Magenta,
    White,
    Gray,
}

impl CategoryColor {
    pub(crate) const ALL: [CategoryColor; 8] = [
        CategoryColor::Red,
        CategoryColor::Yellow,
        CategoryColor::Green,
        CategoryColor::Cyan,
        CategoryColor::Blue,
        CategoryColor::Magenta,
        CategoryColor::White,
        CategoryColor::Gray,
    ];

    pub(crate) fn to_color(self) -> Color {
        match self {
            CategoryColor::Red => Color::Red,
            CategoryColor::Yellow => Color::Yellow,
            CategoryColor::Green => Color::Green,
            CategoryColor::Cyan => Color::Cyan,
            CategoryColor::Blue => Color::Blue,
            CategoryColor::Magenta => Color::Magenta,
            CategoryColor::White => Color::White,
            CategoryColor::Gray => Color::Gray,
        }
    }

    pub(crate) fn label(self) -> &'static str {
        match self {
            CategoryColor::Red => "vermelho",
            CategoryColor::Yellow => "amarelo",
            CategoryColor::Green => "verde",
            CategoryColor::Cyan => "ciano",
            CategoryColor::Blue => "azul",
            CategoryColor::Magenta => "magenta",
            CategoryColor::White => "branco",
            CategoryColor::Gray => "cinza",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Category {
    pub(crate) name: String,
    pub(crate) color: CategoryColor,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Task {
    pub(crate) title: String,
    pub(crate) done: bool,
    #[serde(default = "default_true")]
    pub(crate) active: bool,
    #[serde(default = "Utc::now")]
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) completed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub(crate) tags: Vec<String>,
}

impl Task {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            done: false,
            active: true,
            created_at: Utc::now(),
            completed_at: None,
            tags: Vec::new(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    Inserting,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum AppMode {
    List,
    CategoryEdit,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum CategoryScreenMode {
    Browsing,
    Editing,
    ConfirmDelete,
}

#[derive(Debug)]
pub(crate) struct SlashMenu {
    pub(crate) slash_pos: usize,
    pub(crate) selected: usize,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum FooterAction {
    EnterInsert,
    ToggleDone,
    ToggleActive,
    ToggleShowInactive,
    DeleteSelected,
    EnterCategoryEdit,
    Quit,
    ConfirmNewTask,
    OpenSlashMenu,
    CancelInsert,
    SlashMenuConfirm,
    SlashMenuClose,
    NewCategory,
    EditCategory,
    DeleteCategory,
    LeaveCategoryScreen,
    CategoryColorPrev,
    CategoryColorNext,
    ConfirmCategoryForm,
    CancelCategoryForm,
    ConfirmDeleteCategory,
    CancelDeleteCategory,
}

#[derive(Debug, Clone)]
pub(crate) struct FooterHint {
    pub(crate) area: Rect,
    pub(crate) action: FooterAction,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct LayoutRects {
    pub(crate) task_list: Option<Rect>,
    pub(crate) category_list: Option<Rect>,
    pub(crate) input: Option<Rect>,
    pub(crate) slash_menu: Option<Rect>,
    pub(crate) slash_menu_items: Vec<Rect>,
    pub(crate) color_cells: Vec<Rect>,
    pub(crate) footer_hints: Vec<FooterHint>,
}

pub(crate) struct App {
    pub(crate) tasks: Vec<Task>,
    pub(crate) categories: Vec<Category>,
    pub(crate) list_state: ListState,
    pub(crate) mode: InputMode,
    pub(crate) screen: AppMode,
    pub(crate) input_buffer: String,
    pub(crate) status: Option<String>,
    pub(crate) should_quit: bool,
    pub(crate) show_inactive: bool,

    pub(crate) pending_tags: Vec<String>,
    pub(crate) slash_menu: Option<SlashMenu>,

    pub(crate) category_screen_mode: CategoryScreenMode,
    pub(crate) category_list_state: ListState,
    pub(crate) category_name_buffer: String,
    pub(crate) category_color_index: usize,
    pub(crate) editing_category_index: Option<usize>,

    pub(crate) layout: LayoutRects,
}

impl App {
    pub(crate) fn new(tasks: Vec<Task>, categories: Vec<Category>) -> Self {
        let mut list_state = ListState::default();
        if !tasks.is_empty() {
            list_state.select(Some(0));
        }
        let mut category_list_state = ListState::default();
        if !categories.is_empty() {
            category_list_state.select(Some(0));
        }
        Self {
            tasks,
            categories,
            list_state,
            mode: InputMode::Normal,
            screen: AppMode::List,
            input_buffer: String::new(),
            status: None,
            should_quit: false,
            show_inactive: false,
            pending_tags: Vec::new(),
            slash_menu: None,
            category_screen_mode: CategoryScreenMode::Browsing,
            category_list_state,
            category_name_buffer: String::new(),
            category_color_index: 0,
            editing_category_index: None,
            layout: LayoutRects::default(),
        }
    }

    pub(crate) fn visible_indices(&self) -> Vec<usize> {
        self.tasks
            .iter()
            .enumerate()
            .filter(|(_, t)| self.show_inactive || t.active)
            .map(|(i, _)| i)
            .collect()
    }

    pub(crate) fn selected_task_index(&self) -> Option<usize> {
        let visible = self.visible_indices();
        self.list_state
            .selected()
            .and_then(|i| visible.get(i).copied())
    }

    fn clamp_selection_to_visible(&mut self, prev_visible_pos: usize) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.list_state.select(None);
        } else if prev_visible_pos >= visible.len() {
            self.list_state.select(Some(visible.len() - 1));
        }
    }

    pub(crate) fn select_next(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < visible.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub(crate) fn select_previous(&mut self) {
        let visible = self.visible_indices();
        if visible.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) => visible.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub(crate) fn toggle_selected(&mut self) {
        if let Some(real) = self.selected_task_index() {
            if let Some(task) = self.tasks.get_mut(real) {
                task.done = !task.done;
                task.completed_at = if task.done { Some(Utc::now()) } else { None };
            }
        }
    }

    pub(crate) fn delete_selected(&mut self) {
        let Some(vis_i) = self.list_state.selected() else {
            return;
        };
        let Some(real) = self.selected_task_index() else {
            return;
        };
        self.tasks.remove(real);
        self.clamp_selection_to_visible(vis_i);
    }

    pub(crate) fn toggle_active_selected(&mut self) {
        let Some(vis_i) = self.list_state.selected() else {
            return;
        };
        let Some(real) = self.selected_task_index() else {
            return;
        };
        if let Some(task) = self.tasks.get_mut(real) {
            task.active = !task.active;
        }
        self.clamp_selection_to_visible(vis_i);
    }

    pub(crate) fn toggle_show_inactive(&mut self) {
        let prev_real = self.selected_task_index();
        self.show_inactive = !self.show_inactive;
        let visible = self.visible_indices();
        let fallback = if visible.is_empty() { None } else { Some(0) };
        let new_sel = prev_real
            .and_then(|r| visible.iter().position(|&v| v == r))
            .or(fallback);
        self.list_state.select(new_sel);
    }

    pub(crate) fn enter_insert_mode(&mut self) {
        self.mode = InputMode::Inserting;
        self.input_buffer.clear();
        self.pending_tags.clear();
        self.slash_menu = None;
        self.status = None;
    }

    pub(crate) fn cancel_insert_mode(&mut self) {
        self.mode = InputMode::Normal;
        self.input_buffer.clear();
        self.pending_tags.clear();
        self.slash_menu = None;
    }

    pub(crate) fn confirm_new_task(&mut self) {
        let title = self.input_buffer.trim();
        if !title.is_empty() {
            let mut task = Task::new(title);
            task.tags = std::mem::take(&mut self.pending_tags);
            self.tasks.push(task);
            let new_real = self.tasks.len() - 1;
            let visible = self.visible_indices();
            if let Some(pos) = visible.iter().position(|&v| v == new_real) {
                self.list_state.select(Some(pos));
            }
        }
        self.input_buffer.clear();
        self.pending_tags.clear();
        self.slash_menu = None;
        self.mode = InputMode::Normal;
    }

    pub(crate) fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.active && t.done).count()
    }

    pub(crate) fn active_total(&self) -> usize {
        self.tasks.iter().filter(|t| t.active).count()
    }

    pub(crate) fn progress_ratio(&self) -> f64 {
        let active_total = self.active_total();
        if active_total == 0 {
            0.0
        } else {
            self.done_count() as f64 / active_total as f64
        }
    }

    pub(crate) fn slash_query(&self) -> &str {
        match &self.slash_menu {
            Some(m) if m.slash_pos < self.input_buffer.len() => {
                &self.input_buffer[m.slash_pos + 1..]
            }
            _ => "",
        }
    }

    pub(crate) fn slash_filtered_indices(&self) -> Vec<usize> {
        let q = self.slash_query().to_lowercase();
        self.categories
            .iter()
            .enumerate()
            .filter(|(_, c)| c.name.to_lowercase().contains(&q))
            .map(|(i, _)| i)
            .collect()
    }

    pub(crate) fn open_slash_menu(&mut self) {
        let pos = self.input_buffer.len();
        self.input_buffer.push('/');
        self.slash_menu = Some(SlashMenu {
            slash_pos: pos,
            selected: 0,
        });
    }

    pub(crate) fn slash_menu_select_next(&mut self) {
        let n = self.slash_filtered_indices().len();
        if n == 0 {
            return;
        }
        if let Some(m) = &mut self.slash_menu {
            m.selected = (m.selected + 1) % n;
        }
    }

    pub(crate) fn slash_menu_select_prev(&mut self) {
        let n = self.slash_filtered_indices().len();
        if n == 0 {
            return;
        }
        if let Some(m) = &mut self.slash_menu {
            m.selected = (m.selected + n - 1) % n;
        }
    }

    pub(crate) fn slash_menu_close(&mut self) {
        self.slash_menu = None;
    }

    pub(crate) fn slash_menu_confirm(&mut self) {
        let filtered = self.slash_filtered_indices();
        let menu_selected = match &self.slash_menu {
            Some(m) => m.selected,
            None => return,
        };
        if filtered.is_empty() {
            self.slash_menu = None;
            return;
        }
        let cat_idx = filtered[menu_selected.min(filtered.len() - 1)];
        let cat_name = self.categories[cat_idx].name.clone();
        if let Some(m) = &self.slash_menu {
            self.input_buffer.truncate(m.slash_pos);
        }
        if !self.pending_tags.iter().any(|t| t == &cat_name) {
            self.pending_tags.push(cat_name);
        }
        self.slash_menu = None;
    }

    pub(crate) fn input_push_char(&mut self, c: char) {
        self.input_buffer.push(c);
        self.clamp_slash_selection();
    }

    pub(crate) fn input_pop_char(&mut self) {
        if self.input_buffer.is_empty() {
            return;
        }
        self.input_buffer.pop();
        if let Some(m) = &self.slash_menu {
            if self.input_buffer.len() <= m.slash_pos {
                self.slash_menu = None;
                return;
            }
        }
        self.clamp_slash_selection();
    }

    fn clamp_slash_selection(&mut self) {
        let n = self.slash_filtered_indices().len();
        if let Some(m) = &mut self.slash_menu {
            if n == 0 {
                m.selected = 0;
            } else if m.selected >= n {
                m.selected = n - 1;
            }
        }
    }

    pub(crate) fn enter_category_edit(&mut self) {
        self.screen = AppMode::CategoryEdit;
        self.category_screen_mode = CategoryScreenMode::Browsing;
        self.category_name_buffer.clear();
        self.category_color_index = 0;
        self.editing_category_index = None;
        if !self.categories.is_empty() && self.category_list_state.selected().is_none() {
            self.category_list_state.select(Some(0));
        }
        self.status = None;
    }

    pub(crate) fn leave_category_screen(&mut self) {
        self.screen = AppMode::List;
        self.category_screen_mode = CategoryScreenMode::Browsing;
        self.category_name_buffer.clear();
        self.editing_category_index = None;
        self.status = None;
    }

    pub(crate) fn category_select_next(&mut self) {
        if self.categories.is_empty() {
            self.category_list_state.select(None);
            return;
        }
        let i = match self.category_list_state.selected() {
            Some(i) if i + 1 < self.categories.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.category_list_state.select(Some(i));
    }

    pub(crate) fn category_select_prev(&mut self) {
        if self.categories.is_empty() {
            self.category_list_state.select(None);
            return;
        }
        let i = match self.category_list_state.selected() {
            Some(0) => self.categories.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.category_list_state.select(Some(i));
    }

    pub(crate) fn start_new_category(&mut self) {
        self.category_screen_mode = CategoryScreenMode::Editing;
        self.editing_category_index = None;
        self.category_name_buffer.clear();
        self.category_color_index = 0;
        self.status = None;
    }

    pub(crate) fn start_edit_selected_category(&mut self) {
        let Some(i) = self.category_list_state.selected() else {
            return;
        };
        let Some(cat) = self.categories.get(i) else {
            return;
        };
        self.category_screen_mode = CategoryScreenMode::Editing;
        self.editing_category_index = Some(i);
        self.category_name_buffer = cat.name.clone();
        self.category_color_index = CategoryColor::ALL
            .iter()
            .position(|c| *c == cat.color)
            .unwrap_or(0);
        self.status = None;
    }

    pub(crate) fn cancel_category_form(&mut self) {
        self.category_screen_mode = CategoryScreenMode::Browsing;
        self.category_name_buffer.clear();
        self.editing_category_index = None;
        self.status = None;
    }

    pub(crate) fn confirm_category_form(&mut self) {
        let name = self.category_name_buffer.trim().to_string();
        if name.is_empty() {
            self.status = Some("Nome da categoria não pode estar vazio.".to_string());
            return;
        }
        let color = CategoryColor::ALL[self.category_color_index];

        match self.editing_category_index {
            None => {
                if self
                    .categories
                    .iter()
                    .any(|c| c.name.eq_ignore_ascii_case(&name))
                {
                    self.status = Some(format!("Categoria '{}' já existe.", name));
                    return;
                }
                self.categories.push(Category { name, color });
                self.category_list_state
                    .select(Some(self.categories.len() - 1));
            }
            Some(idx) => {
                if self
                    .categories
                    .iter()
                    .enumerate()
                    .any(|(i, c)| i != idx && c.name.eq_ignore_ascii_case(&name))
                {
                    self.status = Some(format!("Categoria '{}' já existe.", name));
                    return;
                }
                let old_name = self.categories[idx].name.clone();
                if old_name != name {
                    for t in &mut self.tasks {
                        for tag in &mut t.tags {
                            if *tag == old_name {
                                *tag = name.clone();
                            }
                        }
                    }
                }
                self.categories[idx].name = name;
                self.categories[idx].color = color;
            }
        }
        self.category_screen_mode = CategoryScreenMode::Browsing;
        self.category_name_buffer.clear();
        self.editing_category_index = None;
    }

    pub(crate) fn request_delete_category(&mut self) {
        if self.category_list_state.selected().is_some() {
            self.category_screen_mode = CategoryScreenMode::ConfirmDelete;
        }
    }

    pub(crate) fn confirm_delete_category(&mut self) {
        if let Some(idx) = self.category_list_state.selected() {
            if idx < self.categories.len() {
                let removed = self.categories.remove(idx);
                for t in &mut self.tasks {
                    t.tags.retain(|tag| tag != &removed.name);
                }
                if self.categories.is_empty() {
                    self.category_list_state.select(None);
                } else if idx >= self.categories.len() {
                    self.category_list_state
                        .select(Some(self.categories.len() - 1));
                }
            }
        }
        self.category_screen_mode = CategoryScreenMode::Browsing;
    }

    pub(crate) fn cancel_delete_category(&mut self) {
        self.category_screen_mode = CategoryScreenMode::Browsing;
    }

    pub(crate) fn category_color_next(&mut self) {
        let len = CategoryColor::ALL.len();
        self.category_color_index = (self.category_color_index + 1) % len;
    }

    pub(crate) fn category_color_prev(&mut self) {
        let len = CategoryColor::ALL.len();
        self.category_color_index = (self.category_color_index + len - 1) % len;
    }

    pub(crate) fn category_name_push(&mut self, c: char) {
        self.category_name_buffer.push(c);
    }

    pub(crate) fn category_name_pop(&mut self) {
        self.category_name_buffer.pop();
    }

    pub(crate) fn category_color(&self, name: &str) -> Option<CategoryColor> {
        self.categories
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.color)
    }

    pub(crate) fn category_usage_count(&self, name: &str) -> usize {
        self.tasks
            .iter()
            .filter(|t| t.tags.iter().any(|tag| tag == name))
            .count()
    }
}
