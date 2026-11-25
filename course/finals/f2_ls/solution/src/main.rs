use f2_ls::{format_grid, format_long, parse_args, read_entries};
use std::path::Path;

/// 端末の幅を調べる: ioctl(TIOCGWINSZ) → 環境変数 COLUMNS → 80
fn terminal_width() -> usize {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col as usize;
        }
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (mut opts, paths) = match parse_args(&args) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("f2_ls: {e}");
            std::process::exit(2);
        }
    };
    // 出力先が端末のときだけ色を付ける (パイプ先に ESC を流さない)
    opts.color = unsafe { libc::isatty(libc::STDOUT_FILENO) } == 1;
    let width = terminal_width();
    let mut code = 0;
    let multi = paths.len() > 1;
    for (i, p) in paths.iter().enumerate() {
        match read_entries(Path::new(p), opts.all) {
            Ok(entries) => {
                if multi {
                    if i > 0 {
                        println!();
                    }
                    println!("{p}:");
                }
                let lines = if opts.long {
                    format_long(&entries, opts.color)
                } else {
                    format_grid(&entries, width, opts.color)
                };
                for l in lines {
                    println!("{l}");
                }
            }
            Err(e) => {
                eprintln!("f2_ls: {p}: {e}");
                code = 1;
            }
        }
    }
    std::process::exit(code);
}
