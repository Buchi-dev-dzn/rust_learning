//! 文字の表示幅 (セル数)。全角文字は 2、結合文字は 0。
//! (p05_editor と同じ考え方。本格的には East Asian Width 表を使う)

pub fn char_width(c: char) -> usize {
    let cp = c as u32;
    if c.is_control()
        || matches!(cp, 0x0300..=0x036F | 0x200B..=0x200F | 0x3099..=0x309A | 0xFE00..=0xFE0F)
    {
        return 0;
    }
    let wide = matches!(cp,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD
    );
    if wide { 2 } else { 1 }
}

pub fn str_width(s: &str) -> usize {
    s.chars().map(char_width).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths() {
        assert_eq!(char_width('a'), 1);
        assert_eq!(char_width('あ'), 2);
        assert_eq!(char_width('─'), 1);
        assert_eq!(str_width("タスク1"), 7);
    }
}
