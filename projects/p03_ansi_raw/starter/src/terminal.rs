//! 端末に依存する部分 (libc の termios / ioctl)。

use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::sync::Mutex;

use crate::ansi;

/// raw モードにする前の端末設定。復元に使う。パニックフックからも参照するのでグローバルに置く。
static ORIGINAL: Mutex<Option<libc::termios>> = Mutex::new(None);

/// raw モードの間だけ生きるガード。
pub struct RawMode {
    _private: (),
}

impl RawMode {
    pub fn enable() -> io::Result<RawMode> {
        // TODO(STEP 2): tcgetattr で現在の設定を取得し ORIGINAL に保存
        // TODO(STEP 2): c_iflag / c_oflag / c_cflag / c_lflag / c_cc[VMIN, VTIME] を変更して tcsetattr
        // TODO(STEP 3): 代替スクリーンに入り、カーソルを隠す
        let _ = (&ORIGINAL, MaybeUninit::<libc::termios>::uninit(), ansi::ENTER_ALT_SCREEN);
        todo!("RawMode::enable")
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        restore();
    }
}

/// 端末を元に戻す。何度呼んでもよい (2 回目以降は何もしない)。
pub fn restore() {
    // TODO(STEP 2): ORIGINAL から take() して、Some なら tcsetattr で戻す
    // TODO(STEP 3): RESET, SHOW_CURSOR, LEAVE_ALT_SCREEN を出力する
}

/// パニック時に端末を戻してから、標準のパニック表示を行うフックを設定する。
pub fn install_panic_hook() {
    // TODO(STEP 3): std::panic::take_hook / set_hook
}

/// 端末の大きさ (列数, 行数) を ioctl(TIOCGWINSZ) で取得する。
pub fn window_size() -> io::Result<(u16, u16)> {
    // TODO(STEP 4): libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, ...) で libc::winsize を埋める
    Ok((80, 24))
}

/// stdin から読めるだけ読む。VTIME により最大 0.1 秒で戻り、何も無ければ空。
pub fn read_input(buf: &mut [u8]) -> io::Result<usize> {
    // SAFETY: buf は有効な可変スライス
    let n = unsafe { libc::read(libc::STDIN_FILENO, buf.as_mut_ptr().cast(), buf.len()) };
    if n < 0 {
        let err = io::Error::last_os_error();
        if err.kind() == io::ErrorKind::Interrupted || err.kind() == io::ErrorKind::WouldBlock {
            return Ok(0);
        }
        return Err(err);
    }
    let _ = io::stdout().flush();
    Ok(n as usize)
}
