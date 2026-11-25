use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use p02_todo_cli_starter::{execute, parse_command};

fn todo_path() -> PathBuf {
    // TODO(STEP 5): 環境変数 TODO_FILE があればそれを使う
    PathBuf::from("todo.txt")
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    // TODO(STEP 7): unwrap をやめて match し、エラーは stderr へ + 終了コード (使い方の誤りは 2, それ以外は 1)
    let cmd = parse_command(&args).unwrap();
    let msg = execute(&todo_path(), cmd).unwrap();
    println!("{msg}");
    ExitCode::SUCCESS
}
