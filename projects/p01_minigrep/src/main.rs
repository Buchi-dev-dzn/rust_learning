use std::env;
use std::process::ExitCode;

use p01_minigrep::{Config, USAGE, run};

// 終了コードは grep の慣習に合わせる: 0 = 一致あり, 1 = 一致なし, 2 = エラー
fn main() -> ExitCode {
    let ignore_case_env = env::var_os("IGNORE_CASE").is_some();

    let config = match Config::build(env::args().skip(1), ignore_case_env) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("minigrep: {e}");
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };

    match run(&config) {
        Ok(lines) if lines.is_empty() => ExitCode::from(1),
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("minigrep: {e}");
            ExitCode::from(2)
        }
    }
}
