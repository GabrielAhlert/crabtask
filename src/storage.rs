use std::{fs, io, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::app::{Category, Task};

pub(crate) const STORAGE_FILE: &str = "tasks.json";

#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Storage {
    #[serde(default)]
    pub(crate) tasks: Vec<Task>,
    #[serde(default)]
    pub(crate) categories: Vec<Category>,
}

#[derive(Serialize)]
struct StorageRef<'a> {
    tasks: &'a [Task],
    categories: &'a [Category],
}

fn storage_path() -> PathBuf {
    PathBuf::from(STORAGE_FILE)
}

pub(crate) fn load_storage() -> Storage {
    let path = storage_path();
    if !path.exists() {
        return Storage::default();
    }
    let Ok(content) = fs::read_to_string(&path) else {
        return Storage::default();
    };
    if content.trim().is_empty() {
        return Storage::default();
    }
    if let Ok(s) = serde_json::from_str::<Storage>(&content) {
        return s;
    }
    // Fallback: legacy format was a bare Vec<Task>.
    if let Ok(tasks) = serde_json::from_str::<Vec<Task>>(&content) {
        return Storage {
            tasks,
            categories: Vec::new(),
        };
    }
    Storage::default()
}

pub(crate) fn save_storage(tasks: &[Task], categories: &[Category]) -> io::Result<()> {
    let wire = StorageRef { tasks, categories };
    let json = serde_json::to_string_pretty(&wire)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(storage_path(), json)
}
