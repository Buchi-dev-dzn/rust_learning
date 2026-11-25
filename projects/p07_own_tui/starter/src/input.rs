//! raw モードで読んだバイト列をキーイベントに変換する。
//!
//! | 入力                 | バイト列                         |
//! | -------------------- | -------------------------------- |
//! | a                    | 0x61                             |
//! | Ctrl-A .. Ctrl-Z     | 0x01 .. 0x1A (Tab=0x09, Enter=0x0D は別扱い) |
//! | Backspace            | 0x7F (端末によっては 0x08)        |
//! | Esc                  | 0x1B (単独)                      |
//! | ↑ ↓ → ←              | ESC [ A / B / C / D (ESC O A も)  |
//! | Home / End           | ESC [ H / F, ESC [ 1 ~ / 4 ~      |
//! | Delete / PgUp / PgDn | ESC [ 3 ~ / 5 ~ / 6 ~            |
//! | 日本語の文字         | UTF-8 の 2〜4 バイト              |

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    /// Ctrl + 英字 (小文字)
    Ctrl(char),
    Enter,
    Tab,
    BackTab,
    Backspace,
    Esc,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    /// 解釈できなかったシーケンス
    Unknown,
}

/// 1 回の read で得たバイト列を、キーの列に変換する。
///
/// 簡略化のため「1 回の read の中で ESC が末尾に単独で来たら Esc キー」とみなす。
/// (ESC [ A が 2 回の read に分かれて届くと誤判定するが、実用上ほぼ起きない)
pub fn parse_keys(bytes: &[u8]) -> Vec<Key> {
    // TODO(STEP 7): parse_one を繰り返し呼び、消費したバイト数だけ進める。
    let _ = bytes;
    todo!()
}

/// 先頭から 1 キーぶん読み、(キー, 使ったバイト数) を返す
fn parse_one(b: &[u8]) -> (Key, usize) {
    // TODO(STEP 7): ESC / Enter / Tab / Backspace / Ctrl / ASCII / UTF-8 を判別する。
    let _ = b;
    todo!()
}

fn parse_escape(b: &[u8]) -> (Key, usize) {
    match b.get(1) {
        None => (Key::Esc, 1),
        Some(b'[') => parse_csi(b),
        Some(b'O') => {
            // SS3: アプリケーションカーソルキーモードの矢印など
            let key = match b.get(2) {
                Some(b'A') => Key::Up,
                Some(b'B') => Key::Down,
                Some(b'C') => Key::Right,
                Some(b'D') => Key::Left,
                Some(b'H') => Key::Home,
                Some(b'F') => Key::End,
                None => return (Key::Unknown, 2),
                _ => Key::Unknown,
            };
            (key, 3)
        }
        // ESC の後に別のキー (Alt+キー など)。ここでは Esc として扱い、続きは次のキーにする
        Some(_) => (Key::Esc, 1),
    }
}

/// CSI: ESC [ <数字や ;> <終端文字 0x40..=0x7E>
fn parse_csi(b: &[u8]) -> (Key, usize) {
    let mut end = 2;
    while end < b.len() && !(0x40..=0x7e).contains(&b[end]) {
        end += 1;
    }
    if end >= b.len() {
        return (Key::Unknown, b.len()); // 途中で切れている
    }
    let params = std::str::from_utf8(&b[2..end]).unwrap_or("");
    let first: u32 = params.split(';').next().and_then(|p| p.parse().ok()).unwrap_or(0);
    let key = match b[end] {
        b'A' => Key::Up,
        b'B' => Key::Down,
        b'C' => Key::Right,
        b'D' => Key::Left,
        b'H' => Key::Home,
        b'F' => Key::End,
        b'Z' => Key::BackTab,
        b'~' => match first {
            1 | 7 => Key::Home,
            4 | 8 => Key::End,
            3 => Key::Delete,
            5 => Key::PageUp,
            6 => Key::PageDown,
            _ => Key::Unknown,
        },
        _ => Key::Unknown,
    };
    (key, end + 1)
}

/// UTF-8 の先頭バイトから長さを決めて 1 文字にする
fn parse_utf8(b: &[u8]) -> (Key, usize) {
    let len = match b[0] {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => return (Key::Unknown, 1), // 継続バイトが先頭に来た = 壊れている
    };
    if b.len() < len {
        return (Key::Unknown, b.len());
    }
    match std::str::from_utf8(&b[..len]).ok().and_then(|s| s.chars().next()) {
        Some(c) => (Key::Char(c), len),
        None => (Key::Unknown, 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_and_control() {
        assert_eq!(
            parse_keys(b"a\x01\r\t\x7f\x11"),
            [Key::Char('a'), Key::Ctrl('a'), Key::Enter, Key::Tab, Key::Backspace, Key::Ctrl('q')]
        );
    }

    #[test]
    fn arrows_and_special_keys() {
        assert_eq!(
            parse_keys(b"\x1b[A\x1b[B\x1b[C\x1b[D\x1bOA"),
            [Key::Up, Key::Down, Key::Right, Key::Left, Key::Up]
        );
        assert_eq!(
            parse_keys(b"\x1b[3~\x1b[5~\x1b[6~\x1b[1~\x1b[F\x1b[Z"),
            [Key::Delete, Key::PageUp, Key::PageDown, Key::Home, Key::End, Key::BackTab]
        );
        // 修飾キー付き (Ctrl+→ = ESC [ 1 ; 5 C) も矢印として読める
        assert_eq!(parse_keys(b"\x1b[1;5C"), [Key::Right]);
    }

    #[test]
    fn lone_escape() {
        assert_eq!(parse_keys(b"\x1b"), [Key::Esc]);
        assert_eq!(parse_keys(b"\x1bx"), [Key::Esc, Key::Char('x')]);
    }

    #[test]
    fn utf8_chars() {
        assert_eq!(
            parse_keys("あa漢🦀".as_bytes()),
            [Key::Char('あ'), Key::Char('a'), Key::Char('漢'), Key::Char('🦀')]
        );
    }

    #[test]
    fn broken_input_does_not_panic() {
        assert_eq!(parse_keys(&[0xe3, 0x81]), [Key::Unknown]);
        assert_eq!(parse_keys(b"\x1b[12"), [Key::Unknown]);
        assert_eq!(parse_keys(&[0x80, b'a']), [Key::Unknown, Key::Char('a')]);
    }
}
