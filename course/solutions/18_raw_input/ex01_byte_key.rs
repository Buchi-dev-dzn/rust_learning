//! # 1 バイトのキーを分類する
//!
//! raw モードで stdin から読んだ 1 バイトを `Key` に変換する `byte_to_key` と、
//! 逆に「Ctrl + 文字」が何バイトになるかを返す `ctrl_byte` を実装しよう。
//!
//! `byte_to_key` の規則 (上から順に判定):
//! - `0x0D` (`\r`) → `Enter` (raw モードでは Enter は `\n` ではなく `\r`)
//! - `0x09` → `Tab`
//! - `0x7F` または `0x08` → `Backspace`
//! - `0x1B` → `Esc`
//! - `0x01..=0x1A` → `Ctrl(小文字)` (例: `0x03` → `Ctrl('c')`)
//! - `0x20..=0x7E` → `Char(文字)`
//! - それ以外 → `Unknown(b)`
//!
//! `ctrl_byte(c)` は英字 (大文字・小文字どちらでも) に対して `Some(c & 0x1F)`、
//! それ以外は `None` を返す。例: `ctrl_byte('q')` = `Some(0x11)`
//!
//! - 関連: 第18章「バイトを読む」「Ctrl キーの仕組み」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, PartialEq)]
enum Key {
    Char(char),
    Ctrl(char),
    Enter,
    Tab,
    Backspace,
    Esc,
    Unknown(u8),
}

fn byte_to_key(b: u8) -> Key {
    // match の腕は上から順に試されるので、Enter/Tab を Ctrl より先に書く
    match b {
        0x0d => Key::Enter,
        0x09 => Key::Tab,
        0x7f | 0x08 => Key::Backspace,
        0x1b => Key::Esc,
        0x01..=0x1a => Key::Ctrl((b - 1 + b'a') as char),
        0x20..=0x7e => Key::Char(b as char),
        _ => Key::Unknown(b),
    }
}

fn ctrl_byte(c: char) -> Option<u8> {
    if c.is_ascii_alphabetic() {
        Some(c as u8 & 0x1f)
    } else {
        None
    }
}

fn main() {
    for b in [b'a', 0x03, 0x0d, 0x7f, 0x1b, 0x11] {
        println!("{b:#04x} -> {:?}", byte_to_key(b));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn printable() {
        assert_eq!(byte_to_key(b'a'), Key::Char('a'));
        assert_eq!(byte_to_key(b'Z'), Key::Char('Z'));
        assert_eq!(byte_to_key(b' '), Key::Char(' '));
        assert_eq!(byte_to_key(b'~'), Key::Char('~'));
    }

    #[test]
    fn special() {
        assert_eq!(byte_to_key(b'\r'), Key::Enter);
        assert_eq!(byte_to_key(b'\t'), Key::Tab);
        assert_eq!(byte_to_key(0x7f), Key::Backspace);
        assert_eq!(byte_to_key(0x08), Key::Backspace);
        assert_eq!(byte_to_key(0x1b), Key::Esc);
    }

    #[test]
    fn ctrl() {
        assert_eq!(byte_to_key(0x01), Key::Ctrl('a'));
        assert_eq!(byte_to_key(0x03), Key::Ctrl('c'));
        assert_eq!(byte_to_key(0x0a), Key::Ctrl('j')); // \n
        assert_eq!(byte_to_key(0x11), Key::Ctrl('q'));
        assert_eq!(byte_to_key(0x1a), Key::Ctrl('z'));
    }

    #[test]
    fn unknown() {
        assert_eq!(byte_to_key(0x00), Key::Unknown(0x00));
        assert_eq!(byte_to_key(0x1c), Key::Unknown(0x1c));
        assert_eq!(byte_to_key(0xe3), Key::Unknown(0xe3));
    }

    #[test]
    fn encode_ctrl() {
        assert_eq!(ctrl_byte('a'), Some(0x01));
        assert_eq!(ctrl_byte('C'), Some(0x03));
        assert_eq!(ctrl_byte('q'), Some(0x11));
        assert_eq!(ctrl_byte('1'), None);
        for c in 'a'..='z' {
            assert_eq!(byte_to_key(ctrl_byte(c).unwrap()) == Key::Ctrl(c), !"him".contains(c), "{c}");
            // Ctrl-H/I/M は Backspace/Tab/Enter と区別できない
        }
    }
}
