mod app;
mod storage;
mod tui;
mod ui;

use std::{io, panic};

use crate::app::App;
use crate::storage::{load_tasks, save_tasks, STORAGE_FILE};
use crate::tui::{force_restore_terminal, init_terminal, restore_terminal, run_app};

fn main() -> io::Result<()> {
    // Make sure the terminal is restored even if we panic in the middle of drawing.
    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        force_restore_terminal();
        original_hook(info);
    }));

    let tasks = load_tasks();
    let mut app = App::new(tasks);

    let mut terminal = init_terminal()?;
    let run_result = run_app(&mut terminal, &mut app);
    let restore_result = restore_terminal(&mut terminal);

    // Always try to persist whatever state we have, even after a partial error.
    let save_result = save_tasks(&app.tasks);

    run_result?;
    restore_result?;
    save_result?;

    println!(
        "CrabTask: {} tarefa(s) salva(s) em {}",
        app.tasks.len(),
        STORAGE_FILE
    );
    Ok(())
}
