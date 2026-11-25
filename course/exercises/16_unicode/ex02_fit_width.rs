//! # 列幅に合わせて切り詰め・埋める
//!
//! TUI の表やリストでは、文字列を「ちょうど N 列」に収める処理が頻出する。
//! 表示幅を考えて次の 3 つの関数を実装しよう (`char_width` は用意済み)。
//!
//! - `truncate(s, max)` … 表示幅が `max` 以下になる最長の先頭部分を `&str` で返す
//!   (全角文字が境界をまたぐならその文字は入れない)
//! - `fit(s, width)` … 結果の表示幅が **ちょうど** `width` になる `String` を返す
//!   - 収まるなら右側を空白で埋める (左寄せ)
//!   - 収まらないなら `width - 1` 列以内に切り詰めて `…` (幅 1) を付け、足りない分は空白で埋める
//!   - `width == 0` なら空文字列
//! - `fit_right(s, width)` … `fit` の右寄せ版 (左側を空白で埋める。はみ出す場合は `fit` と同じ結果)
//!
//! - 関連: 第16章「幅に合わせて埋める・切り詰める」
//!
//! テスト (`mod tests`) は編集しないこと。

/// 簡易版の表示幅 (かな・漢字・全角英数は 2、それ以外は 1)
fn char_width(c: char) -> usize {
    let cp = c as u32;
    if matches!(cp, 0x3000..=0x9FFF | 0xFF01..=0xFF60) { 2 } else { 1 }
}

fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

// TODO: char_indices() で幅を足していき、max を超える文字の手前のバイト位置で切る
fn truncate(s: &str, max: usize) -> &str {
    todo!()
}

// TODO: ちょうど width 列になるように整える
fn fit(s: &str, width: usize) -> String {
    todo!()
}

// TODO: 右寄せ版
fn fit_right(s: &str, width: usize) -> String {
    todo!()
}

fn main() {
    for name in ["main.rs", "日本語ファイル名.txt", "ａｂｃ"] {
        println!("|{}|{}|", fit(name, 10), fit_right(name, 8));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_ascii() {
        assert_eq!(truncate("hello", 3), "hel");
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello", 0), "");
    }

    #[test]
    fn truncate_wide() {
        assert_eq!(truncate("あいう", 4), "あい");
        assert_eq!(truncate("あいう", 5), "あい"); // 'う' は 2 列なので入らない
        assert_eq!(truncate("aあb", 2), "a");
        assert_eq!(truncate("aあb", 3), "aあ");
    }

    #[test]
    fn fit_pads() {
        assert_eq!(fit("ab", 5), "ab   ");
        assert_eq!(fit("あ", 4), "あ  ");
        assert_eq!(fit("abc", 3), "abc");
    }

    #[test]
    fn fit_truncates_with_ellipsis() {
        assert_eq!(fit("abcdef", 4), "abc…");
        assert_eq!(fit("あいうえ", 5), "あい…");
        assert_eq!(fit("あいうえ", 4), "あ… "); // 'い' が入らない分は空白
        assert_eq!(fit("abc", 1), "…");
        assert_eq!(fit("abc", 0), "");
    }

    #[test]
    fn fit_always_exact_width() {
        for s in ["", "a", "あいう", "Rust言語のTUI", "ａｂｃｄｅ"] {
            for w in 0..12 {
                assert_eq!(str_width(&fit(s, w)), w, "fit({s:?}, {w})");
                assert_eq!(str_width(&fit_right(s, w)), w, "fit_right({s:?}, {w})");
            }
        }
    }

    #[test]
    fn fit_right_aligns() {
        assert_eq!(fit_right("42", 5), "   42");
        assert_eq!(fit_right("漢字", 6), "  漢字");
        assert_eq!(fit_right("abcdef", 4), "abc…");
    }
}
