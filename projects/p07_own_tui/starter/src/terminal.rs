//! 端末そのものを扱う部分 (libc のみ)。
//!
//! - `RawMode`  : termios で raw モードにし、Drop で元に戻すガード
//! - `size`     : ioctl(TIOCGWINSZ) で端末の大きさを得る
//! - `Terminal` : 代替スクリーン + ダブルバッファリングによる描画
//! - `Events`   : poll() でタイムアウト付きに入力を待ち、キーやリサイズに変換する

use std::io::{self, Write};
use std::sync::Mutex;
use std::time::Duration;

use crate::buffer::Buffer;
use crate::geometry::Rect;
use crate::input::{Key, parse_keys};
use crate::render::diff_to_ansi;

/// raw モードに入る前の設定。パニックフックからも戻せるように static に置く
static ORIGINAL: Mutex<Option<libc::termios>> = Mutex::new(None);

pub struct RawMode {
    _private: (),
}

impl RawMode {
    pub fn enable() -> io::Result<Self> {
        unsafe {
            let mut orig: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(libc::STDIN_FILENO, &mut orig) != 0 {
                return Err(io::Error::last_os_error());
            }
            let mut raw = orig;
            // 入力: Ctrl-S/Q のフロー制御 (IXON) と CR→LF 変換 (ICRNL) などを止める
            raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
            // 出力: \n → \r\n の変換を止める (カーソル位置は自分で指定する)
            raw.c_oflag &= !libc::OPOST;
            raw.c_cflag |= libc::CS8;
            // エコー・行単位入力・Ctrl-V・Ctrl-C/Z のシグナルを止める
            raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
            // read は 1 バイト来たら返る (待つのは poll に任せる)
            raw.c_cc[libc::VMIN] = 1;
            raw.c_cc[libc::VTIME] = 0;
            if libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &raw) != 0 {
                return Err(io::Error::last_os_error());
            }
            *ORIGINAL.lock().unwrap() = Some(orig);
        }
        Ok(RawMode { _private: () })
    }
}

/// raw モードを解除する (何度呼んでもよい)
pub fn disable_raw_mode() {
    let orig = ORIGINAL.lock().map(|mut g| g.take()).unwrap_or(None);
    if let Some(orig) = orig {
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &orig);
        }
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        disable_raw_mode();
    }
}

/// 端末の (幅, 高さ)。取得できなければ 80x24
pub fn size() -> (u16, u16) {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            (ws.ws_col, ws.ws_row)
        } else {
            (80, 24)
        }
    }
}

const ENTER_SCREEN: &str = "\x1b[?1049h\x1b[?25l\x1b[2J";
const LEAVE_SCREEN: &str = "\x1b[0m\x1b[?25h\x1b[?1049l";

/// 画面を元に戻す: スタイルをリセット、カーソル表示、代替スクリーンを抜ける、raw モード解除
pub fn restore() {
    let mut out = io::stdout();
    let _ = out.write_all(LEAVE_SCREEN.as_bytes());
    let _ = out.flush();
    disable_raw_mode();
}

/// 描画担当。前フレームのバッファを覚えておき、差分だけを出力する
pub struct Terminal {
    _raw: RawMode,
    prev: Buffer,
    /// 直近の draw で出力したバイト数 (差分描画の効果を見る用)
    pub last_output_len: usize,
}

impl Terminal {
    pub fn new() -> io::Result<Self> {
        // パニックしたら、メッセージを出す前に端末を戻す
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore();
            default_hook(info);
        }));

        let raw = RawMode::enable()?;
        let mut out = io::stdout();
        out.write_all(ENTER_SCREEN.as_bytes())?;
        out.flush()?;
        let (w, h) = size();
        Ok(Terminal { _raw: raw, prev: Buffer::empty(Rect::new(0, 0, w, h)), last_output_len: 0 })
    }

    pub fn area(&self) -> Rect {
        self.prev.area
    }

    /// `f` に空のバッファを渡して描かせ、前フレームとの差分だけを端末に送る
    pub fn draw(&mut self, f: impl FnOnce(&mut Buffer)) -> io::Result<()> {
        let (w, h) = size();
        let area = Rect::new(0, 0, w, h);
        let mut out = String::new();
        if area != self.prev.area {
            // サイズが変わったら画面を消して、全セルを描き直す
            out.push_str("\x1b[2J");
            self.prev = Buffer::empty(area);
        }
        let mut next = Buffer::empty(area);
        f(&mut next);
        out.push_str(&diff_to_ansi(&next.diff(&self.prev)));
        self.last_output_len = out.len();
        let mut stdout = io::stdout();
        stdout.write_all(out.as_bytes())?;
        stdout.flush()?;
        self.prev = next;
        Ok(())
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let mut out = io::stdout();
        let _ = out.write_all(LEAVE_SCREEN.as_bytes());
        let _ = out.flush();
        // raw モードの解除は _raw (RawMode) の Drop が行う
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Event {
    Key(Key),
    Resize(u16, u16),
    /// タイムアウトまで何も起きなかった (アニメーションや時計の更新に使う)
    Tick,
}

/// 入力イベントの取り出し口
pub struct Events {
    last_size: (u16, u16),
}

impl Default for Events {
    fn default() -> Self {
        Self::new()
    }
}

impl Events {
    pub fn new() -> Self {
        Events { last_size: size() }
    }

    /// 最大 `timeout` だけ入力を待ち、起きたイベントを返す (空になることはない)
    pub fn next(&mut self, timeout: Duration) -> io::Result<Vec<Event>> {
        let mut events = Vec::new();
        let mut fds = libc::pollfd { fd: libc::STDIN_FILENO, events: libc::POLLIN, revents: 0 };
        let ms = timeout.as_millis().min(i32::MAX as u128) as i32;
        let n = unsafe { libc::poll(&mut fds, 1, ms) };
        if n < 0 {
            let err = io::Error::last_os_error();
            // リサイズ (SIGWINCH) などのシグナルで中断されただけならエラーにしない
            if err.kind() != io::ErrorKind::Interrupted {
                return Err(err);
            }
        } else if n > 0 {
            let mut buf = [0u8; 1024];
            let len = unsafe { libc::read(libc::STDIN_FILENO, buf.as_mut_ptr().cast(), buf.len()) };
            if len < 0 {
                return Err(io::Error::last_os_error());
            }
            if len == 0 {
                // 入力が閉じられた (EOF)。Ctrl-D と同じ扱いにしてアプリに終了を促す
                events.push(Event::Key(Key::Ctrl('d')));
            }
            events.extend(parse_keys(&buf[..len as usize]).into_iter().map(Event::Key));
        }
        let now = size();
        if now != self.last_size {
            self.last_size = now;
            events.push(Event::Resize(now.0, now.1));
        }
        if events.is_empty() {
            events.push(Event::Tick);
        }
        Ok(events)
    }
}
