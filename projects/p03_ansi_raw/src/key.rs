//! 入力バイト列をキーに変換するパーサ。
//!
//! raw モードの端末からは、押したキーが「バイト列」として届く。
//!
//! | キー        | バイト列                |
//! |-------------|-------------------------|
//! | `a`         | `61`                    |
//! | `あ`        | `e3 81 82` (UTF-8)      |
//! | Enter       | `0d`                    |
//! | Ctrl-A..Z   | `01`..`1a`              |
//! | Esc         | `1b`                    |
//! | ↑ ↓ → ←     | `1b 5b 41` .. `1b 5b 44` (`ESC [ A` ..) |
//! | Delete      | `1b 5b 33 7e` (`ESC [ 3 ~`) |
//! | Alt-x       | `1b 78` (`ESC x`)       |

/// 解析済みのキー。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    /// Ctrl + 英字。中身は小文字 (`Ctrl('c')` = Ctrl-C)。
    Ctrl(char),
    Alt(char),
    Enter,
    Tab,
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
    /// 解釈できなかったバイト列
    Unknown(Vec<u8>),
}

const ESC: u8 = 0x1b;

/// `input` の先頭から 1 キー分を解析し、(キー, 消費したバイト数) を返す。
/// `input` が空なら `None`。
///
/// 1 回の `read` で届いたバイト列を渡す前提で、末尾にある単独の `ESC` は Esc キーとみなす。
pub fn parse_key(input: &[u8]) -> Option<(Key, usize)> {
    let &first = input.first()?;
    let parsed = match first {
        ESC => parse_escape(input),
        b'\r' | b'\n' => (Key::Enter, 1),
        b'\t' => (Key::Tab, 1),
        0x7f | 0x08 => (Key::Backspace, 1),
        // 0x01..=0x1a は Ctrl-A..Ctrl-Z (Tab=Ctrl-I, Enter=Ctrl-M, BS=Ctrl-H は上で処理済み)
        0x01..=0x1a => (Key::Ctrl((b'a' + first - 1) as char), 1),
        0x20..=0x7e => (Key::Char(first as char), 1),
        0x80..=0xff => parse_utf8(input),
        _ => (Key::Unknown(vec![first]), 1),
    };
    Some(parsed)
}

/// バイト列全体をキーの列に変換する。
pub fn parse_keys(mut input: &[u8]) -> Vec<Key> {
    let mut keys = Vec::new();
    while let Some((key, n)) = parse_key(input) {
        keys.push(key);
        input = &input[n..];
    }
    keys
}

fn parse_escape(input: &[u8]) -> (Key, usize) {
    match input.get(1) {
        None => (Key::Esc, 1),
        Some(b'[') => parse_csi(input),
        // SS3: 一部の端末は矢印キーを ESC O A のように送る
        Some(b'O') => match input.get(2) {
            Some(b'A') => (Key::Up, 3),
            Some(b'B') => (Key::Down, 3),
            Some(b'C') => (Key::Right, 3),
            Some(b'D') => (Key::Left, 3),
            Some(b'H') => (Key::Home, 3),
            Some(b'F') => (Key::End, 3),
            _ => (Key::Alt('O'), 2),
        },
        Some(&b) if (0x20..=0x7e).contains(&b) => (Key::Alt(b as char), 2),
        // ESC ESC など: 最初の ESC だけを Esc として消費する
        Some(_) => (Key::Esc, 1),
    }
}

/// `ESC [ <パラメータ> <終端文字>` を解析する。
/// パラメータは数字と `;`、終端文字は 0x40..=0x7e の 1 バイト。
fn parse_csi(input: &[u8]) -> (Key, usize) {
    // input[0] = ESC, input[1] = '['
    let Some(end) = input[2..]
        .iter()
        .position(|b| (0x40..=0x7e).contains(b))
        .map(|p| p + 2)
    else {
        // 終端文字が来ていない (途中で切れた) → 全部まとめて Unknown
        return (Key::Unknown(input.to_vec()), input.len());
    };
    let params = &input[2..end];
    let len = end + 1;
    let key = match (params, input[end]) {
        (_, b'A') => Key::Up,
        (_, b'B') => Key::Down,
        (_, b'C') => Key::Right,
        (_, b'D') => Key::Left,
        (_, b'H') => Key::Home,
        (_, b'F') => Key::End,
        (b"1" | b"7", b'~') => Key::Home,
        (b"4" | b"8", b'~') => Key::End,
        (b"3", b'~') => Key::Delete,
        (b"5", b'~') => Key::PageUp,
        (b"6", b'~') => Key::PageDown,
        _ => Key::Unknown(input[..len].to_vec()),
    };
    (key, len)
}

/// UTF-8 の先頭バイトから文字の長さを求めて 1 文字を取り出す。
fn parse_utf8(input: &[u8]) -> (Key, usize) {
    let len = match input[0] {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => return (Key::Unknown(vec![input[0]]), 1),
    };
    if input.len() < len {
        return (Key::Unknown(input.to_vec()), input.len());
    }
    match std::str::from_utf8(&input[..len]) {
        Ok(s) => (Key::Char(s.chars().next().unwrap()), len),
        Err(_) => (Key::Unknown(vec![input[0]]), 1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn one(bytes: &[u8]) -> Key {
        let (k, n) = parse_key(bytes).unwrap();
        assert_eq!(n, bytes.len(), "全バイトを消費するはず: {bytes:?}");
        k
    }

    #[test]
    fn empty_is_none() {
        assert_eq!(parse_key(b""), None);
    }

    #[test]
    fn ascii_and_control() {
        assert_eq!(one(b"a"), Key::Char('a'));
        assert_eq!(one(b" "), Key::Char(' '));
        assert_eq!(one(b"\r"), Key::Enter);
        assert_eq!(one(b"\t"), Key::Tab);
        assert_eq!(one(&[0x7f]), Key::Backspace);
        assert_eq!(one(&[0x01]), Key::Ctrl('a'));
        assert_eq!(one(&[0x03]), Key::Ctrl('c'));
        assert_eq!(one(&[0x11]), Key::Ctrl('q'));
        assert_eq!(one(&[0x1a]), Key::Ctrl('z'));
    }

    #[test]
    fn arrows_csi_and_ss3() {
        assert_eq!(one(b"\x1b[A"), Key::Up);
        assert_eq!(one(b"\x1b[B"), Key::Down);
        assert_eq!(one(b"\x1b[C"), Key::Right);
        assert_eq!(one(b"\x1b[D"), Key::Left);
        assert_eq!(one(b"\x1bOA"), Key::Up);
        // 修飾キー付き (Shift+↑ = ESC [ 1 ; 2 A) も矢印として扱う
        assert_eq!(one(b"\x1b[1;2A"), Key::Up);
    }

    #[test]
    fn tilde_keys() {
        assert_eq!(one(b"\x1b[3~"), Key::Delete);
        assert_eq!(one(b"\x1b[5~"), Key::PageUp);
        assert_eq!(one(b"\x1b[6~"), Key::PageDown);
        assert_eq!(one(b"\x1b[1~"), Key::Home);
        assert_eq!(one(b"\x1b[4~"), Key::End);
        assert_eq!(one(b"\x1b[H"), Key::Home);
        assert_eq!(one(b"\x1b[99~"), Key::Unknown(b"\x1b[99~".to_vec()));
    }

    #[test]
    fn esc_and_alt() {
        assert_eq!(one(b"\x1b"), Key::Esc);
        assert_eq!(one(b"\x1bx"), Key::Alt('x'));
        assert_eq!(parse_keys(b"\x1b\x1b"), vec![Key::Esc, Key::Esc]);
    }

    #[test]
    fn utf8() {
        assert_eq!(one("あ".as_bytes()), Key::Char('あ'));
        assert_eq!(one("é".as_bytes()), Key::Char('é'));
        assert_eq!(one("🦀".as_bytes()), Key::Char('🦀'));
        // 途中で切れた UTF-8
        assert_eq!(one(&[0xe3, 0x81]), Key::Unknown(vec![0xe3, 0x81]));
    }

    #[test]
    fn several_keys_in_one_read() {
        assert_eq!(
            parse_keys(b"ab\x1b[A\x1b[3~q\x03"),
            vec![
                Key::Char('a'),
                Key::Char('b'),
                Key::Up,
                Key::Delete,
                Key::Char('q'),
                Key::Ctrl('c'),
            ]
        );
        assert_eq!(parse_keys("日本".as_bytes()), vec![Key::Char('日'), Key::Char('本')]);
    }

    #[test]
    fn truncated_csi() {
        assert_eq!(one(b"\x1b[1;"), Key::Unknown(b"\x1b[1;".to_vec()));
    }
}
