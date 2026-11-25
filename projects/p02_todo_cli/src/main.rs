use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use p02_todo_cli::{TodoError, execute, parse_command};

fn todo_path() -> PathBuf {
    env::var_os("TODO_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("todo.txt"))
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    let result = parse_command(&args).and_then(|cmd| execute(&todo_path(), cmd));

    match result {
        Ok(msg) => {
            println!("{msg}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("todo: {e}");
            // 使い方の誤りは 2、それ以外の失敗は 1
            match e {
                TodoError::Usage(_) | TodoError::InvalidId(_) => ExitCode::from(2),
                _ => ExitCode::FAILURE,
            }
        }
    }
}
