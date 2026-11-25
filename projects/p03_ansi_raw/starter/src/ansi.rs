//! ANSI エスケープシーケンス。
//!
//! `ESC` (0x1b) + `[` で始まるものを CSI (Control Sequence Introducer) と呼ぶ。
//! 端末はこれを文字として表示せず、命令として解釈する。

/// 画面全体を消去する。
pub const CLEAR_SCREEN: &str = "\x1b[2J";
/// カーソルから行末までを消去する。
pub const CLEAR_LINE: &str = "\x1b[K";
/// カーソルを隠す / 表示する。
pub const HIDE_CURSOR: &str = "\x1b[?25l";
pub const SHOW_CURSOR: &str = "\x1b[?25h";
/// 代替スクリーン (alternate screen) に入る / 出る。
pub const ENTER_ALT_SCREEN: &str = "\x1b[?1049h";
pub const LEAVE_ALT_SCREEN: &str = "\x1b[?1049l";
/// 文字装飾をすべて解除する。
pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const REVERSE: &str = "\x1b[7m";

/// カーソルを (row, col) に移動する。引数は 0 始まり、ANSI の座標は 1 始まり。
pub fn move_to(row: u16, col: u16) -> String {
    // TODO(STEP 1): ESC [ 行 ; 列 H を返す (どちらも +1 する)
    let _ = (row, col);
    todo!("move_to")
}

/// 前景色 (文字色) を 256 色パレットから設定する。
pub fn fg(color: u8) -> String {
    // TODO(STEP 1): ESC [ 38 ; 5 ; 色番号 m
    let _ = color;
    todo!("fg")
}

/// 背景色を 256 色パレットから設定する。
pub fn bg(color: u8) -> String {
    // TODO(STEP 1): ESC [ 48 ; 5 ; 色番号 m
    let _ = color;
    todo!("bg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step1_move_to_is_one_based() {
        assert_eq!(move_to(0, 0), "\x1b[1;1H");
        assert_eq!(move_to(4, 9), "\x1b[5;10H");
    }

    #[test]
    fn step1_colors() {
        assert_eq!(fg(196), "\x1b[38;5;196m");
        assert_eq!(bg(21), "\x1b[48;5;21m");
    }
}
