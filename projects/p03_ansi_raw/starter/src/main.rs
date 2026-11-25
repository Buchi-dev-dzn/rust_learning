use std::io::{self, Write};
use std::process::ExitCode;

use p03_ansi_raw_starter::app::App;
use p03_ansi_raw_starter::key::parse_key;
use p03_ansi_raw_starter::terminal::{self, RawMode};

fn run() -> io::Result<()> {
    // TODO(STEP 3): terminal::install_panic_hook() を呼ぶ
    let (cols, rows) = terminal::window_size().unwrap_or((80, 24));
    let _raw = RawMode::enable()?; // ここから先、どう抜けても Drop で元に戻る

    let mut app = App::new(cols, rows);
    let mut out = io::stdout();
    let mut buf = [0u8; 64];
    let mut dirty = true; // 再描画が必要か

    while !app.quit {
        if dirty {
            out.write_all(app.render().as_bytes())?;
            out.flush()?;
            dirty = false;
        }

        // 入力待ち (最大 0.1 秒)。届いたバイト列をキーに分解する
        let n = terminal::read_input(&mut buf)?;
        let mut input = &buf[..n];
        while let Some((key, used)) = parse_key(input) {
            app.last_bytes = input[..used].to_vec();
            app.handle_key(key);
            input = &input[used..];
            dirty = true;
        }

        // TODO(STEP 7): terminal::window_size() を毎回呼び、変わっていたら app.resize して dirty = true
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("ansi_raw: {e}");
            eprintln!("(端末 (TTY) の中で実行してください)");
            ExitCode::FAILURE
        }
    }
}
