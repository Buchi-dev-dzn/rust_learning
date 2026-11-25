use std::env;
use std::process::ExitCode;

use p01_minigrep_starter::{Config, USAGE, run};

fn main() -> ExitCode {
    // TODO(STEP 4): 環境変数 IGNORE_CASE が設定されていれば true にする
    let ignore_case_env = false;

    // TODO(STEP 2): Err のときは eprintln! でエラーと USAGE を出し、ExitCode::from(2) を返す
    let config = Config::build(env::args().skip(1), ignore_case_env).unwrap();

    // TODO(STEP 6): 一致なしなら 1、エラーなら stderr に出して 2 を返す
    let lines = run(&config).unwrap();
    for line in lines {
        println!("{line}");
    }
    let _ = USAGE;
    ExitCode::SUCCESS
}
