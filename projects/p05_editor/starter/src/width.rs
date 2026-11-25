//! 文字の「表示幅」(ターミナルで何セル使うか) を求める。
//!
//! 外部クレートなしで、よく使う範囲だけを手書きで判定する。

/// 1 文字が端末上で占めるセル数 (0, 1, 2) を返す。
pub fn char_width(c: char) -> usize {
    // TODO(STEP 2): 結合文字 (U+0300..=U+036F など) は 0、
    //               is_wide(c as u32) なら 2、それ以外は 1 を返す
    todo!()
}

#[allow(dead_code)]
fn is_wide(cp: u32) -> bool {
    // TODO(STEP 2): ひらがな・カタカナ (0x3041..=0x33FF)、CJK 漢字 (0x4E00..=0x9FFF)、
    //               全角英数 (0xFF00..=0xFF60) などの範囲なら true
    todo!()
}

/// 文字列全体の表示幅
pub fn str_width(s: &str) -> usize {
    // TODO(STEP 2): char_width の合計
    todo!()
}

/// 表示幅が `max` を超えないように先頭から切り詰める
pub fn truncate_to_width(s: &str, max: usize) -> String {
    // TODO(STEP 2): 1 文字ずつ幅を足していき、max を超える直前で止める
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_1() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(str_width("hello"), 5);
    }

    #[test]
    fn japanese_is_2() {
        assert_eq!(char_width('あ'), 2);
        assert_eq!(char_width('漢'), 2);
        assert_eq!(char_width('Ａ'), 2);
        assert_eq!(str_width("日本語abc"), 9);
    }

    #[test]
    fn combining_is_0() {
        assert_eq!(str_width("e\u{0301}"), 1);
    }

    #[test]
    fn truncate_respects_wide_chars() {
        assert_eq!(truncate_to_width("あいう", 5), "あい");
        assert_eq!(truncate_to_width("abc", 10), "abc");
    }
}
