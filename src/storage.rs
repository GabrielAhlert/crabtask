use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

use crate::app::{Category, Task};

const STORAGE_FILE_NAME: &str = "tasks.json";
const ENV_OVERRIDE: &str = "CRABTASK_FILE";

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

/// Resolve the storage file path with priority: CLI override > env > XDG/platform default.
/// Falls back to `./tasks.json` only if the platform dirs cannot be determined.
pub(crate) fn resolve_storage_path(cli_override: Option<PathBuf>) -> PathBuf {
    if let Some(p) = cli_override {
        return p;
    }
    if let Ok(p) = env::var(ENV_OVERRIDE) {
        if !p.is_empty() {
            return PathBuf::from(p);
        }
    }
    if let Some(pd) = ProjectDirs::from("", "", "crabtask") {
        return pd.data_dir().join(STORAGE_FILE_NAME);
    }
    PathBuf::from(STORAGE_FILE_NAME)
}

pub(crate) fn load_storage(path: &Path) -> Storage {
    if !path.exists() {
        return Storage::default();
    }
    let Ok(content) = fs::read_to_string(path) else {
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

pub(crate) fn save_storage(path: &Path, tasks: &[Task], categories: &[Category]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    let wire = StorageRef { tasks, categories };
    let json = serde_json::to_string_pretty(&wire)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    fs::write(path, json)
}
