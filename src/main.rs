mod app;
mod storage;
mod tui;
mod ui;

use std::{io, panic, path::PathBuf, process};

use crate::app::App;
use crate::storage::{load_storage, resolve_storage_path, save_storage};
use crate::tui::{force_restore_terminal, init_terminal, restore_terminal, run_app};

const USAGE: &str = "\
crabtask — TUI To-Do em Rust

USO:
    crabtask [OPÇÕES]

OPÇÕES:
    -f, --file <PATH>    Caminho do arquivo de dados (default: pasta de dados do SO)
        --path           Mostra o caminho resolvido do arquivo de dados e sai
    -h, --help           Mostra esta ajuda
    -V, --version        Mostra a versão

VARIÁVEIS DE AMBIENTE:
    CRABTASK_FILE        Mesmo efeito de --file, com prioridade menor que a flag.

PRIORIDADE: --file > CRABTASK_FILE > local padrão do sistema operacional.
";

fn main() -> io::Result<()> {
    let cli_file = match parse_args() {
        Ok(action) => match action {
            CliAction::Run { file } => file,
            CliAction::PrintPath { file } => {
                let path = resolve_storage_path(file);
                println!("{}", path.display());
                return Ok(());
            }
            CliAction::PrintHelp => {
                print!("{}", USAGE);
                return Ok(());
            }
            CliAction::PrintVersion => {
                println!("crabtask {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
        },
        Err(msg) => {
            eprintln!("crabtask: {}", msg);
            eprintln!("tente `crabtask --help`");
            process::exit(2);
        }
    };

    let storage_path = resolve_storage_path(cli_file);

    let original_hook = panic::take_hook();
    panic::set_hook(Box::new(move |info| {
        force_restore_terminal();
        original_hook(info);
    }));

    let storage = load_storage(&storage_path);
    let mut app = App::new(storage.tasks, storage.categories);

    let mut terminal = init_terminal()?;
    let run_result = run_app(&mut terminal, &mut app);
    let restore_result = restore_terminal(&mut terminal);

    let save_result = save_storage(&storage_path, &app.tasks, &app.categories);

    run_result?;
    restore_result?;
    save_result?;

    println!(
        "CrabTask: {} tarefa(s) e {} categoria(s) salva(s) em {}",
        app.tasks.len(),
        app.categories.len(),
        storage_path.display()
    );
    Ok(())
}

enum CliAction {
    Run { file: Option<PathBuf> },
    PrintPath { file: Option<PathBuf> },
    PrintHelp,
    PrintVersion,
}

fn parse_args() -> Result<CliAction, String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut file: Option<PathBuf> = None;
    let mut print_path = false;
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-f" | "--file" => {
                let value = args
                    .get(i + 1)
                    .ok_or_else(|| format!("{} requer um caminho como argumento", args[i]))?;
                file = Some(PathBuf::from(value));
                i += 2;
            }
            "--path" => {
                print_path = true;
                i += 1;
            }
            "-h" | "--help" => return Ok(CliAction::PrintHelp),
            "-V" | "--version" => return Ok(CliAction::PrintVersion),
            other => return Err(format!("argumento desconhecido: {}", other)),
        }
    }
    Ok(if print_path {
        CliAction::PrintPath { file }
    } else {
        CliAction::Run { file }
    })
}
