use f2_ls::{format_grid, format_long, parse_args, read_entries};
use std::path::Path;

/// 端末の幅を調べる。
fn terminal_width() -> usize {
    // TODO: libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) で ws.ws_col を得る。
    //       失敗したら環境変数 COLUMNS、それも無ければ 80。
    80
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
    // TODO: 標準出力が端末のときだけ色を付ける (libc::isatty)
    opts.color = false;
    let width = terminal_width();
    let mut code = 0;
    // TODO: パスが複数なら "PATH:" の見出しと空行を入れる
    for p in &paths {
        match read_entries(Path::new(p), opts.all) {
            Ok(entries) => {
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
