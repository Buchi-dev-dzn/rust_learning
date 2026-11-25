//! # カーソルと画面制御のシーケンス
//!
//! ANSI エスケープシーケンスを組み立てる関数を実装しよう。
//! 戻り値は `String` なので、端末がなくても文字列として確かめられる。
//!
//! - `move_to(x, y)` … 0 始まりの (x, y) へ移動。端末は 1 始まりの `ESC [ 行 ; 列 H`
//! - `move_by(dx, dy)` … 相対移動。dy<0 なら上 `A`、dy>0 なら下 `B`、dx>0 なら右 `C`、dx<0 なら左 `D`。
//!   縦 → 横の順に連結し、0 の方向は何も出さない。例: `move_by(-2, 3)` = `"\x1b[3B\x1b[2D"`
//! - `clear_screen()` … 画面全体を消去して左上へ: `"\x1b[2J\x1b[H"`
//! - `clear_line()` … 現在行を全消去: `"\x1b[2K"`
//! - `enter_tui()` … 代替スクリーンへ切替 → カーソル非表示 → 画面消去 (`clear_screen()` の内容)
//! - `leave_tui()` … カーソル表示 → 通常スクリーンへ戻る
//!
//! - 関連: 第17章「カーソル移動」「消去」「代替スクリーンとカーソル表示」
//!
//! テスト (`mod tests`) は編集しないこと。

const ESC: &str = "\x1b";

fn move_to(x: u16, y: u16) -> String {
    // y が先、しかも 1 始まり
    format!("{ESC}[{};{}H", y + 1, x + 1)
}

fn move_by(dx: i32, dy: i32) -> String {
    let mut s = String::new();
    if dy < 0 {
        s += &format!("{ESC}[{}A", -dy);
    } else if dy > 0 {
        s += &format!("{ESC}[{dy}B");
    }
    if dx > 0 {
        s += &format!("{ESC}[{dx}C");
    } else if dx < 0 {
        s += &format!("{ESC}[{}D", -dx);
    }
    s
}

fn clear_screen() -> String {
    format!("{ESC}[2J{ESC}[H")
}

fn clear_line() -> String {
    format!("{ESC}[2K")
}

fn enter_tui() -> String {
    format!("{ESC}[?1049h{ESC}[?25l{}", clear_screen())
}

fn leave_tui() -> String {
    format!("{ESC}[?25h{ESC}[?1049l")
}

fn main() {
    println!("{:?}", move_to(4, 2));
    println!("{:?}", move_by(-2, 3));
    println!("{:?}", enter_tui());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absolute_move() {
        assert_eq!(move_to(0, 0), "\x1b[1;1H");
        assert_eq!(move_to(4, 2), "\x1b[3;5H");
        assert_eq!(move_to(79, 23), "\x1b[24;80H");
    }

    #[test]
    fn relative_move() {
        assert_eq!(move_by(0, 0), "");
        assert_eq!(move_by(0, -1), "\x1b[1A");
        assert_eq!(move_by(0, 5), "\x1b[5B");
        assert_eq!(move_by(3, 0), "\x1b[3C");
        assert_eq!(move_by(-2, 3), "\x1b[3B\x1b[2D");
        assert_eq!(move_by(7, -4), "\x1b[4A\x1b[7C");
    }

    #[test]
    fn clears() {
        assert_eq!(clear_screen(), "\x1b[2J\x1b[H");
        assert_eq!(clear_line(), "\x1b[2K");
    }

    #[test]
    fn tui_setup_and_teardown() {
        assert_eq!(enter_tui(), "\x1b[?1049h\x1b[?25l\x1b[2J\x1b[H");
        assert_eq!(leave_tui(), "\x1b[?25h\x1b[?1049l");
    }
}
