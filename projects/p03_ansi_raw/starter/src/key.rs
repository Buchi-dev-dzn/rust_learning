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
    // TODO(STEP 5): 先頭バイトで分岐する
    //   Enter (\r \n) / Tab / Backspace (0x7f, 0x08) / Ctrl-A..Z (0x01..=0x1a)
    //   / ASCII 印字可能文字 (0x20..=0x7e) / UTF-8 (0x80..) → parse_utf8
    // TODO(STEP 6): ESC (0x1b) → parse_escape
    let _ = (first, ESC);
    todo!("parse_key")
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

/// ESC で始まる入力を解析する。
#[allow(dead_code)]
fn parse_escape(input: &[u8]) -> (Key, usize) {
    // TODO(STEP 6): input.get(1) で分岐
    //   None → Esc / Some(b'[') → parse_csi / Some(b'O') → SS3 (ESC O A など)
    //   印字可能文字 → Alt(c) / それ以外 → Esc (1 バイトだけ消費)
    let _ = input;
    todo!("parse_escape")
}

/// `ESC [ <パラメータ> <終端文字>` を解析する。終端文字は 0x40..=0x7e の 1 バイト。
#[allow(dead_code)]
fn parse_csi(input: &[u8]) -> (Key, usize) {
    // TODO(STEP 6): 終端文字の位置を探し、(パラメータ, 終端文字) の組で match する
    let _ = input;
    todo!("parse_csi")
}

/// UTF-8 の先頭バイトから文字の長さを求めて 1 文字を取り出す。
#[allow(dead_code)]
fn parse_utf8(input: &[u8]) -> (Key, usize) {
    // TODO(STEP 5): 0xc0..=0xdf → 2 バイト, 0xe0..=0xef → 3, 0xf0..=0xf7 → 4
    //   足りなければ Unknown、std::str::from_utf8 で文字に変換
    let _ = input;
    todo!("parse_utf8")
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
