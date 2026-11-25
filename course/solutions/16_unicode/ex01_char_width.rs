//! # 文字の表示幅を求める
//!
//! 端末上で 1 文字が何列を占めるかを返す `char_width` と、文字列全体の幅を返す
//! `str_width` を実装しよう。
//!
//! ルール (簡易版 East Asian Width):
//! - 制御文字 (U+0000〜U+001F, U+007F〜U+009F) … 0
//! - 結合文字 U+0300〜U+036F、ゼロ幅 U+200B〜U+200F、異体字セレクタ U+FE0F … 0
//! - 次の範囲は 2 (全角)
//!   - U+1100〜U+115F (ハングル字母)
//!   - U+2E80〜U+303E (CJK 記号・句読点)
//!   - U+3041〜U+33FF (かな・CJK 互換)
//!   - U+3400〜U+4DBF, U+4E00〜U+9FFF (漢字)
//!   - U+AC00〜U+D7A3 (ハングル音節)
//!   - U+F900〜U+FAFF (CJK 互換漢字)
//!   - U+FF00〜U+FF60, U+FFE0〜U+FFE6 (全角英数・記号)
//!   - U+1F300〜U+1F64F, U+1F900〜U+1F9FF (絵文字)
//!   - U+20000〜U+3FFFD (CJK 拡張)
//! - それ以外 … 1 (半角カナ U+FF61〜U+FF9F も 1)
//!
//! - 関連: 第16章「表示幅と East Asian Width」
//!
//! テスト (`mod tests`) は編集しないこと。

fn char_width(c: char) -> usize {
    let cp = c as u32;
    if cp < 0x20 || (0x7F..=0x9F).contains(&cp) {
        return 0;
    }
    if (0x0300..=0x036F).contains(&cp) || (0x200B..=0x200F).contains(&cp) || cp == 0xFE0F {
        return 0;
    }
    // matches! と範囲パターンで表を素直に書くのが読みやすい
    let wide = matches!(cp,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD
    );
    if wide { 2 } else { 1 }
}

fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

fn main() {
    for s in ["hello", "こんにちは", "ｱｲｳ", "🦀"] {
        println!("{s}: {}", str_width(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_is_one() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width(' '), 1);
        assert_eq!(char_width('~'), 1);
    }

    #[test]
    fn control_is_zero() {
        assert_eq!(char_width('\n'), 0);
        assert_eq!(char_width('\t'), 0);
        assert_eq!(char_width('\x1b'), 0);
        assert_eq!(char_width('\u{7f}'), 0);
    }

    #[test]
    fn cjk_is_two() {
        for c in ['あ', 'ア', '漢', '。', '「', '한', 'Ａ', '！', '￥', '𠮷'] {
            assert_eq!(char_width(c), 2, "{c:?}");
        }
    }

    #[test]
    fn halfwidth_kana_is_one() {
        assert_eq!(char_width('ｱ'), 1);
        assert_eq!(char_width('ﾟ'), 1);
    }

    #[test]
    fn emoji_and_zero_width() {
        assert_eq!(char_width('🦀'), 2);
        assert_eq!(char_width('😀'), 2);
        assert_eq!(char_width('\u{301}'), 0);
        assert_eq!(char_width('\u{200D}'), 0);
        assert_eq!(char_width('\u{FE0F}'), 0);
    }

    #[test]
    fn other_is_one() {
        assert_eq!(char_width('é'), 1);
        assert_eq!(char_width('─'), 1);
        assert_eq!(char_width('α'), 1);
    }

    #[test]
    fn string_width() {
        assert_eq!(str_width(""), 0);
        assert_eq!(str_width("Rust言語"), 8);
        assert_eq!(str_width("e\u{301}"), 1);
        assert_eq!(str_width("ｱｲｳabc"), 6);
        assert_eq!(str_width("a\tb"), 2);
    }
}
