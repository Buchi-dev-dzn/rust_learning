//! 文字の「表示幅」(ターミナルで何セル使うか) を求める。
//!
//! 本格的には Unicode の East Asian Width 表 (unicode-width クレート) を使うが、
//! ここでは外部クレートなしで、よく使う範囲だけを手書きで判定する。

/// 1 文字が端末上で占めるセル数 (0, 1, 2) を返す。
/// タブはここでは扱わない (位置によって幅が変わるため Row 側で処理する)。
pub fn char_width(c: char) -> usize {
    let cp = c as u32;
    // 結合文字・ゼロ幅文字・異体字セレクタ
    if matches!(cp,
        0x0300..=0x036F   // 結合用ダイアクリティカルマーク
        | 0x200B..=0x200F // ゼロ幅スペース等
        | 0x3099..=0x309A // 結合用濁点・半濁点
        | 0xFE00..=0xFE0F // 異体字セレクタ
    ) {
        return 0;
    }
    if is_wide(cp) { 2 } else { 1 }
}

fn is_wide(cp: u32) -> bool {
    matches!(cp,
        0x1100..=0x115F       // ハングル字母
        | 0x2E80..=0x303E     // CJK 部首・記号と句読点 (「、。」など)
        | 0x3041..=0x33FF     // ひらがな・カタカナ・CJK 互換
        | 0x3400..=0x4DBF     // CJK 統合漢字拡張 A
        | 0x4E00..=0x9FFF     // CJK 統合漢字
        | 0xA000..=0xA4CF     // イ文字
        | 0xAC00..=0xD7A3     // ハングル音節
        | 0xF900..=0xFAFF     // CJK 互換漢字
        | 0xFE30..=0xFE4F     // CJK 互換形
        | 0xFF00..=0xFF60     // 全角英数・記号
        | 0xFFE0..=0xFFE6     // 全角記号
        | 0x1F300..=0x1F64F   // 絵文字
        | 0x1F900..=0x1F9FF   // 絵文字 (補助)
        | 0x20000..=0x3FFFD   // CJK 統合漢字拡張 B 以降
    )
}

/// 文字列全体の表示幅
pub fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

/// 表示幅が `max` を超えないように先頭から切り詰める
pub fn truncate_to_width(s: &str, max: usize) -> String {
    let mut out = String::new();
    let mut w = 0;
    for c in s.chars() {
        let cw = char_width(c);
        if w + cw > max {
            break;
        }
        w += cw;
        out.push(c);
    }
    out
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
