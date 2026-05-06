use chrono::{DateTime, Utc};
use ratatui::widgets::ListState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Task {
    pub(crate) title: String,
    pub(crate) done: bool,
    #[serde(default = "Utc::now")]
    pub(crate) created_at: DateTime<Utc>,
    #[serde(default)]
    pub(crate) completed_at: Option<DateTime<Utc>>,
}

impl Task {
    pub(crate) fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            done: false,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InputMode {
    Normal,
    Inserting,
}

pub(crate) struct App {
    pub(crate) tasks: Vec<Task>,
    pub(crate) list_state: ListState,
    pub(crate) mode: InputMode,
    pub(crate) input_buffer: String,
    pub(crate) status: Option<String>,
    pub(crate) should_quit: bool,
}

impl App {
    pub(crate) fn new(tasks: Vec<Task>) -> Self {
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
}
