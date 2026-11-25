//! 差分 (変わったセルの一覧) を、最小限の ANSI エスケープシーケンスに変換する。
//!
//! 工夫は 2 つだけ:
//! 1. 直前に書いたセルのすぐ右なら、カーソル移動 (`ESC[y;xH`) を省く (端末は書くと自動で右に進む)
//! 2. スタイルが直前と同じなら、SGR (`ESC[...m`) を省く

use crate::buffer::Cell;
use crate::style::Style;
use crate::width::char_width;

pub fn diff_to_ansi(updates: &[(u16, u16, &Cell)]) -> String {
    // TODO(STEP 4):
    //   let mut cursor: Option<(u16, u16)> = None; // 次に書くと文字が置かれる位置の予想
    //   let mut style: Option<Style> = None;       // 直前に出した SGR
    //   各セルについて:
    //     - cursor が (x, y) でなければ "\x1b[{y+1};{x+1}H" (ANSI は 1 始まり・行が先)
    //     - style が違えば cell.style.to_sgr()
    //     - 文字を出して、cursor を文字の幅だけ右へ
    //   何か出力したなら最後に "\x1b[0m"
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;
    use crate::geometry::Rect;
    use crate::style::Color;

    #[test]
    fn contiguous_cells_need_one_move() {
        let area = Rect::new(0, 0, 10, 3);
        let prev = Buffer::empty(area);
        let mut next = Buffer::empty(area);
        next.set_string(2, 1, "abc", Style::new());
        let out = diff_to_ansi(&next.diff(&prev));
        assert_eq!(out, "\x1b[2;3H\x1b[0mabc\x1b[0m");
    }

    #[test]
    fn gaps_and_style_changes() {
        let area = Rect::new(0, 0, 10, 1);
        let prev = Buffer::empty(area);
        let mut next = Buffer::empty(area);
        let red = Style::new().fg(Color::Red);
        next.set_string(0, 0, "a", red);
        next.set_string(1, 0, "b", red);
        next.set_string(5, 0, "c", Style::new());
        let out = diff_to_ansi(&next.diff(&prev));
        assert_eq!(out, "\x1b[1;1H\x1b[0;31mab\x1b[1;6H\x1b[0mc\x1b[0m");
    }

    #[test]
    fn wide_char_advances_two_columns() {
        let area = Rect::new(0, 0, 10, 1);
        let prev = Buffer::empty(area);
        let mut next = Buffer::empty(area);
        next.set_string(0, 0, "あx", Style::new());
        let out = diff_to_ansi(&next.diff(&prev));
        assert_eq!(out, "\x1b[1;1H\x1b[0mあx\x1b[0m");
    }

    #[test]
    fn nothing_changed_means_no_output() {
        let buf = Buffer::empty(Rect::new(0, 0, 3, 3));
        assert_eq!(diff_to_ansi(&buf.diff(&buf)), "");
    }
}
