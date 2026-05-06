use chrono::{DateTime, Utc};
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Task {
    pub(crate) title: String,
    pub(crate) done: bool,
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
pub(crate) enum CategoryFocus {
    Name,
    Color,
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

    pub(crate) category_name_buffer: String,
    pub(crate) category_color_index: usize,
    pub(crate) category_focus: CategoryFocus,
}

impl App {
    pub(crate) fn new(tasks: Vec<Task>, categories: Vec<Category>) -> Self {
        let mut list_state = ListState::default();
        if !tasks.is_empty() {
            list_state.select(Some(0));
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
            category_name_buffer: String::new(),
            category_color_index: 0,
            category_focus: CategoryFocus::Name,
        }
    }

    pub(crate) fn select_next(&mut self) {
        if self.tasks.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(i) if i + 1 < self.tasks.len() => i + 1,
            Some(_) => 0,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub(crate) fn select_previous(&mut self) {
        if self.tasks.is_empty() {
            self.list_state.select(None);
            return;
        }
        let i = match self.list_state.selected() {
            Some(0) => self.tasks.len() - 1,
            Some(i) => i - 1,
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub(crate) fn toggle_selected(&mut self) {
        if let Some(i) = self.list_state.selected() {
            if let Some(task) = self.tasks.get_mut(i) {
                task.done = !task.done;
                task.completed_at = if task.done { Some(Utc::now()) } else { None };
            }
        }
    }

    pub(crate) fn delete_selected(&mut self) {
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

    pub(crate) fn enter_insert_mode(&mut self) {
        self.mode = InputMode::Inserting;
        self.input_buffer.clear();
        self.status = None;
    }

    pub(crate) fn cancel_insert_mode(&mut self) {
        self.mode = InputMode::Normal;
        self.input_buffer.clear();
    }

    pub(crate) fn confirm_new_task(&mut self) {
        let title = self.input_buffer.trim();
        if !title.is_empty() {
            self.tasks.push(Task::new(title));
            self.list_state.select(Some(self.tasks.len() - 1));
        }
        self.input_buffer.clear();
        self.mode = InputMode::Normal;
    }

    pub(crate) fn pending_count(&self) -> usize {
        self.tasks.iter().filter(|t| !t.done).count()
    }

    pub(crate) fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.done).count()
    }

    pub(crate) fn progress_ratio(&self) -> f64 {
        if self.tasks.is_empty() {
            0.0
        } else {
            self.done_count() as f64 / self.tasks.len() as f64
        }
    }

    pub(crate) fn enter_category_edit(&mut self) {
        self.screen = AppMode::CategoryEdit;
        self.category_name_buffer.clear();
        self.category_color_index = 0;
        self.category_focus = CategoryFocus::Name;
        self.status = None;
    }

    pub(crate) fn cancel_category_edit(&mut self) {
        self.screen = AppMode::List;
        self.category_name_buffer.clear();
        self.category_focus = CategoryFocus::Name;
        self.status = None;
    }

    pub(crate) fn confirm_category_edit(&mut self) {
        let name = self.category_name_buffer.trim().to_string();
        if name.is_empty() {
            self.status = Some("Nome da categoria não pode estar vazio.".to_string());
            return;
        }
        if self
            .categories
            .iter()
            .any(|c| c.name.eq_ignore_ascii_case(&name))
        {
            self.status = Some(format!("Categoria '{}' já existe.", name));
            return;
        }
        let color = CategoryColor::ALL[self.category_color_index];
        self.categories.push(Category { name, color });
        self.cancel_category_edit();
    }

    pub(crate) fn category_toggle_focus(&mut self) {
        self.category_focus = match self.category_focus {
            CategoryFocus::Name => CategoryFocus::Color,
            CategoryFocus::Color => CategoryFocus::Name,
        };
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

    pub(crate) fn toggle_tag_on_selected(&mut self, index: usize) {
        let Some(category) = self.categories.get(index) else {
            return;
        };
        let cat_name = category.name.clone();
        if let Some(i) = self.list_state.selected() {
            if let Some(task) = self.tasks.get_mut(i) {
                if let Some(pos) = task.tags.iter().position(|t| t == &cat_name) {
                    task.tags.remove(pos);
                } else {
                    task.tags.push(cat_name);
                }
            }
        }
    }

    pub(crate) fn category_color(&self, name: &str) -> Option<CategoryColor> {
        self.categories
            .iter()
            .find(|c| c.name == name)
            .map(|c| c.color)
    }
}
