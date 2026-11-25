//! 1 行分のテキスト。
//!
//! 位置の単位が 3 種類あることに注意:
//! - `cx`   : 文字 (char) 単位のインデックス。編集操作はこれを使う。
//! - バイト : `String` の内部インデックス。`cx` から `byte_index` で変換する。
//! - `rx`   : 画面上の列 (表示幅の合計)。タブは次のタブストップまで、全角は 2 列。

use crate::width::char_width;

pub const TAB_STOP: usize = 4;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Row {
    pub chars: String,
}

/// 位置 `rx` にある文字 `c` の表示幅
fn width_at(c: char, rx: usize) -> usize {
    if c == '\t' {
        TAB_STOP - (rx % TAB_STOP)
    } else if c.is_control() {
        1 // 制御文字は '?' として 1 列で表示する
    } else {
        char_width(c)
    }
}

impl Row {
    pub fn new(s: impl Into<String>) -> Self {
        Row { chars: s.into() }
    }

    /// 文字数
    pub fn len(&self) -> usize {
        self.chars.chars().count()
    }

    pub fn is_empty(&self) -> bool {
        self.chars.is_empty()
    }

    /// 文字インデックス → バイトインデックス
    pub fn byte_index(&self, cx: usize) -> usize {
        self.chars
            .char_indices()
            .nth(cx)
            .map(|(i, _)| i)
            .unwrap_or(self.chars.len())
    }

    /// 文字インデックス → 画面上の列
    pub fn cx_to_rx(&self, cx: usize) -> usize {
        let mut rx = 0;
        for c in self.chars.chars().take(cx) {
            rx += width_at(c, rx);
        }
        rx
    }

    /// 画面上の列 → 文字インデックス (その列を含む文字)
    pub fn rx_to_cx(&self, target: usize) -> usize {
        let mut rx = 0;
        for (cx, c) in self.chars.chars().enumerate() {
            let w = width_at(c, rx);
            if rx + w > target {
                return cx;
            }
            rx += w;
        }
        self.len()
    }

    /// `cx` の位置にある文字の表示幅 (行末なら 1)
    pub fn width_of(&self, cx: usize) -> usize {
        let rx = self.cx_to_rx(cx);
        self.chars
            .chars()
            .nth(cx)
            .map(|c| width_at(c, rx).max(1))
            .unwrap_or(1)
    }

    pub fn insert_char(&mut self, cx: usize, c: char) {
        let at = self.byte_index(cx);
        self.chars.insert(at, c);
    }

    pub fn delete_char(&mut self, cx: usize) {
        if cx < self.len() {
            let at = self.byte_index(cx);
            self.chars.remove(at);
        }
    }

    pub fn append(&mut self, s: &str) {
        self.chars.push_str(s);
    }

    /// `cx` 以降を切り取って返す (改行の挿入に使う)
    pub fn split_off(&mut self, cx: usize) -> String {
        let at = self.byte_index(cx);
        self.chars.split_off(at)
    }

    /// `coloff` 列目から `width` 列分だけ、画面に出す文字列を作る。
    /// 結果の表示幅は必ず `width` 以下になる。
    /// 左右の端で全角文字が半分だけはみ出す場合は空白で埋める。
    pub fn render(&self, coloff: usize, width: usize) -> String {
        let end = coloff + width;
        let mut out = String::new();
        let mut rx = 0;
        for c in self.chars.chars() {
            let w = width_at(c, rx);
            let start = rx;
            rx += w;
            if w == 0 {
                // 結合文字は直前の文字が表示されていれば一緒に出す
                if start > coloff && start <= end && !out.is_empty() {
                    out.push(c);
                }
                continue;
            }
            if rx <= coloff {
                continue; // 完全に左にはみ出している
            }
            if start >= end {
                break; // 右端を超えた
            }
            let fully_visible = start >= coloff && rx <= end;
            if c == '\t' || !fully_visible {
                // タブ、または端で切れる文字は見えている部分を空白で埋める
                let visible = rx.min(end) - start.max(coloff);
                out.extend(std::iter::repeat_n(' ', visible));
            } else if c.is_control() {
                out.push('?');
            } else {
                out.push(c);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::width::str_width;

    #[test]
    fn cx_rx_ascii() {
        let r = Row::new("hello");
        assert_eq!(r.cx_to_rx(3), 3);
        assert_eq!(r.rx_to_cx(3), 3);
    }

    #[test]
    fn cx_rx_wide() {
        let r = Row::new("aあb");
        assert_eq!(r.cx_to_rx(0), 0);
        assert_eq!(r.cx_to_rx(1), 1);
        assert_eq!(r.cx_to_rx(2), 3);
        assert_eq!(r.cx_to_rx(3), 4);
        // 「あ」の右半分 (rx=2) は「あ」の位置 (cx=1)
        assert_eq!(r.rx_to_cx(2), 1);
        assert_eq!(r.rx_to_cx(3), 2);
        assert_eq!(r.rx_to_cx(100), 3);
    }

    #[test]
    fn tabs_expand_to_tab_stop() {
        let r = Row::new("a\tb");
        assert_eq!(r.cx_to_rx(2), TAB_STOP);
        assert_eq!(r.render(0, 10), "a   b");
    }

    #[test]
    fn insert_and_delete_utf8() {
        let mut r = Row::new("あいう");
        r.insert_char(1, 'X');
        assert_eq!(r.chars, "あXいう");
        r.delete_char(2);
        assert_eq!(r.chars, "あXう");
        let rest = r.split_off(1);
        assert_eq!(r.chars, "あ");
        assert_eq!(rest, "Xう");
    }

    #[test]
    fn render_clips_wide_chars_at_edges() {
        let r = Row::new("あいう");
        // 1 列目から 4 列: 「あ」の右半分(空白) + 「い」 + 「う」の左半分(空白)
        let s = r.render(1, 4);
        assert_eq!(s, " い ");
        assert_eq!(str_width(&s), 4);
        assert_eq!(r.render(0, 3), "あ ");
        assert_eq!(r.render(10, 5), "");
    }

    #[test]
    fn control_chars_are_shown_as_question_mark() {
        let r = Row::new("a\u{1}b");
        assert_eq!(r.render(0, 10), "a?b");
    }
}
