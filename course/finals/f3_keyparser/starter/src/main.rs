//! 押したキーを 1 行ずつ表示するデモ。q または Ctrl-C で終了。

use f3_keyparser::{Key, Parser};
use std::io::{Read, Write};

/// raw モードを有効にし、Drop で元に戻すガード
struct RawMode {
    orig: Option<libc::termios>,
}

impl RawMode {
    fn enable() -> RawMode {
        unsafe {
            let mut t: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut t) != 0 {
                return RawMode { orig: None }; // 端末でない (パイプ等)
            }
            let orig = t;
            libc::cfmakeraw(&mut t);
            // TCSANOW: 既に届いている入力は捨てない
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &t);
            RawMode { orig: Some(orig) }
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        print!("\x1b[?2004l");
        let _ = std::io::stdout().flush();
        if let Some(orig) = self.orig {
            unsafe {
                libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &orig);
            }
        }
    }
}

/// stdin が読めるようになるまで最大 timeout_ms 待つ
fn wait_readable(timeout_ms: i32) -> bool {
    let mut fd = libc::pollfd {
        fd: libc::STDIN_FILENO,
        events: libc::POLLIN,
        revents: 0,
    };
    unsafe { libc::poll(&mut fd, 1, timeout_ms) > 0 }
}

fn main() {
    let _raw = RawMode::enable();
    let mut out = std::io::stdout();
    // ブラケットペーストを有効化
    let _ = write!(out, "\x1b[?2004hpress keys (q / Ctrl-C to quit)\r\n");
    let _ = out.flush();

    let mut parser = Parser::new();
    let mut stdin = std::io::stdin();
    let mut buf = [0u8; 1024];
    'outer: loop {
        let keys = if wait_readable(50) {
            match stdin.read(&mut buf) {
                Ok(0) | Err(_) => break, // EOF
                Ok(n) => parser.feed(&buf[..n]),
            }
        } else if parser.has_pending() {
            // 50ms 続きが来なければ、単独の ESC などとして確定させる
            parser.flush()
        } else {
            continue;
        };
        for k in keys {
            let _ = write!(out, "{k:?}\r\n");
            if k == Key::Char('q') || k == Key::Ctrl('c') {
                break 'outer;
            }
        }
        let _ = out.flush();
    }
}
