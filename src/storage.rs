use std::{fs, io, path::PathBuf};

use crate::app::Task;

pub(crate) const STORAGE_FILE: &str = "tasks.json";

fn storage_path() -> PathBuf {
    PathBuf::from(STORAGE_FILE)
}

pub(crate) fn load_tasks() -> Vec<Task> {
    let path = storage_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) if content.trim().is_empty() => Vec::new(),
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => Vec::new(),
    }
}

pub(crate) fn save_tasks(tasks: &[Task]) -> io::Result<()> {
    let json = serde_json::to_string_pretty(tasks)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(storage_path(), json)
}
