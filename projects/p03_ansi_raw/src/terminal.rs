//! 端末に依存する部分 (libc の termios / ioctl)。
//!
//! - `RawMode::enable()` で raw モード + 代替スクリーン + カーソル非表示にする。
//!   戻り値のガードが Drop されると元に戻る (早期 return や `?` でも確実に)。
//! - `install_panic_hook()` でパニック時にも元に戻す。
//!   パニックのメッセージは unwind (Drop) より **前** に表示されるので、
//!   Drop だけだとメッセージが代替スクリーンに書かれて消えてしまう。

use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::sync::Mutex;

use crate::ansi;

/// raw モードにする前の端末設定。復元に使う。
/// パニックフックからも参照するので、グローバルに置く。
static ORIGINAL: Mutex<Option<libc::termios>> = Mutex::new(None);

/// raw モードの間だけ生きるガード。
pub struct RawMode {
    _private: (),
}

impl RawMode {
    pub fn enable() -> io::Result<RawMode> {
        let fd = libc::STDIN_FILENO;

        // 1. 現在の設定を取得 (stdin が端末でなければここで失敗する)
        let mut orig = MaybeUninit::<libc::termios>::uninit();
        // SAFETY: tcgetattr は成功すると orig を初期化する。戻り値で成否を確認する。
        if unsafe { libc::tcgetattr(fd, orig.as_mut_ptr()) } != 0 {
            return Err(io::Error::last_os_error());
        }
        // SAFETY: 直前の tcgetattr が成功している
        let orig = unsafe { orig.assume_init() };

        // 2. raw 用の設定を作る (cfmakeraw と同等のことを手で書く)
        let mut raw = orig;
        // 入力: Ctrl-S/Q のフロー制御 (IXON)、CR→NL 変換 (ICRNL) などを止める
        raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
        // 出力: "\n" → "\r\n" の変換 (OPOST) を止める
        raw.c_oflag &= !libc::OPOST;
        raw.c_cflag |= libc::CS8;
        // ローカル: エコー、行バッファ (ICANON)、Ctrl-V (IEXTEN)、Ctrl-C/Z のシグナル (ISIG) を止める
        raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
        // read は 1 バイトも来なくても 0.1 秒 (VTIME=1) で戻る → ループで定期処理ができる
        raw.c_cc[libc::VMIN] = 0;
        raw.c_cc[libc::VTIME] = 1;

        // 3. 適用。先に元の設定を保存しておく
        *ORIGINAL.lock().unwrap() = Some(orig);
        // SAFETY: raw は有効な termios 構造体
        if unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &raw) } != 0 {
            let err = io::Error::last_os_error();
            ORIGINAL.lock().unwrap().take();
            return Err(err);
        }

        // 4. 代替スクリーンに入り、カーソルを隠す
        let mut out = io::stdout();
        write!(out, "{}{}", ansi::ENTER_ALT_SCREEN, ansi::HIDE_CURSOR)?;
        out.flush()?;

        Ok(RawMode { _private: () })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        restore();
    }
}

/// 端末を元に戻す。何度呼んでもよい (2 回目以降は何もしない)。
pub fn restore() {
    // パニック中に Mutex が poison されていても中身は取り出す
    let orig = match ORIGINAL.lock() {
        Ok(mut g) => g.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    if let Some(orig) = orig {
        let mut out = io::stdout();
        let _ = write!(
            out,
            "{}{}{}",
            ansi::RESET,
            ansi::SHOW_CURSOR,
            ansi::LEAVE_ALT_SCREEN
        );
        let _ = out.flush();
        // SAFETY: orig は tcgetattr で取得した有効な設定
        unsafe {
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &orig);
        }
    }
}

/// パニック時に端末を戻してから、標準のパニック表示を行うフックを設定する。
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();
        default_hook(info);
    }));
}

/// 端末の大きさ (列数, 行数) を ioctl(TIOCGWINSZ) で取得する。
pub fn window_size() -> io::Result<(u16, u16)> {
    let mut ws = MaybeUninit::<libc::winsize>::zeroed();
    // SAFETY: TIOCGWINSZ は winsize 構造体へのポインタを受け取る
    let r = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ws.as_mut_ptr()) };
    if r != 0 {
        return Err(io::Error::last_os_error());
    }
    // SAFETY: zeroed で初期化済み、かつ ioctl が成功している
    let ws = unsafe { ws.assume_init() };
    if ws.ws_col == 0 || ws.ws_row == 0 {
        return Err(io::Error::other("端末サイズが 0 です"));
    }
    Ok((ws.ws_col, ws.ws_row))
}

/// stdin から読めるだけ読む。VTIME により最大 0.1 秒で戻り、何も無ければ空。
pub fn read_input(buf: &mut [u8]) -> io::Result<usize> {
    // SAFETY: buf は有効な可変スライス
    let n = unsafe { libc::read(libc::STDIN_FILENO, buf.as_mut_ptr().cast(), buf.len()) };
    if n < 0 {
        let err = io::Error::last_os_error();
        // シグナル (端末のリサイズ SIGWINCH など) で中断されただけなら 0 バイト扱い
        if err.kind() == io::ErrorKind::Interrupted || err.kind() == io::ErrorKind::WouldBlock {
            return Ok(0);
        }
        return Err(err);
    }
    Ok(n as usize)
}
