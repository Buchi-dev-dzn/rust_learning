//! 1 行分のテキスト。
//!
//! 位置の単位が 3 種類あることに注意:
//! - `cx`   : 文字 (char) 単位のインデックス。編集操作はこれを使う。
//! - バイト : `String` の内部インデックス。`cx` から `byte_index` で変換する。
//! - `rx`   : 画面上の列 (表示幅の合計)。タブは次のタブストップまで、全角は 2 列。

#[allow(unused_imports)]
use crate::width::char_width;

pub const TAB_STOP: usize = 4;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Row {
    pub chars: String,
}

/// 位置 `rx` にある文字 `c` の表示幅
#[allow(dead_code)]
fn width_at(c: char, rx: usize) -> usize {
    // TODO(STEP 2): タブなら TAB_STOP - (rx % TAB_STOP)、制御文字なら 1、それ以外は char_width(c)
    todo!()
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
        // TODO(STEP 2): char_indices().nth(cx) を使う。行末なら self.chars.len()
        todo!()
    }

    /// 文字インデックス → 画面上の列
    pub fn cx_to_rx(&self, cx: usize) -> usize {
        // TODO(STEP 2): 先頭 cx 文字の width_at を足していく
        todo!()
    }

    /// 画面上の列 → 文字インデックス (その列を含む文字)
    pub fn rx_to_cx(&self, target: usize) -> usize {
        // TODO(STEP 2): rx + w > target になった文字の cx を返す。最後まで来たら len()
        todo!()
    }

    /// `cx` の位置にある文字の表示幅 (行末なら 1)
    pub fn width_of(&self, cx: usize) -> usize {
        // TODO(STEP 4): 横スクロールで全角文字を画面内に収めるのに使う
        todo!()
    }

    pub fn insert_char(&mut self, cx: usize, c: char) {
        // TODO(STEP 5): byte_index で変換してから String::insert
        todo!()
    }

    pub fn delete_char(&mut self, cx: usize) {
        // TODO(STEP 5): cx < len() のときだけ String::remove
        todo!()
    }

    pub fn append(&mut self, s: &str) {
        self.chars.push_str(s);
    }

    /// `cx` 以降を切り取って返す (改行の挿入に使う)
    pub fn split_off(&mut self, cx: usize) -> String {
        // TODO(STEP 5): String::split_off(byte_index(cx))
        todo!()
    }

    /// `coloff` 列目から `width` 列分だけ、画面に出す文字列を作る。
    /// 結果の表示幅は必ず `width` 以下になる。
    pub fn render(&self, coloff: usize, width: usize) -> String {
        // TODO(STEP 2): 1 文字ずつ [start, start + w) の範囲を計算し、
        //   - 完全に左 (rx <= coloff) ならスキップ
        //   - 右端を超えたら break
        //   - タブや、端で半分切れる全角文字は見えている列数ぶんの空白
        //   - 制御文字は '?'
        todo!()
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
