//! F3: キー入力パーサの模範解答。
//!
//! 方針: バッファ先頭から「キー 1 個分」を切り出す `parse_one` を書き、
//! それを繰り返すだけ。バイトが足りなければ `Incomplete` を返して次の `feed` を待つ。

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Ctrl(char),
    Alt(char),
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
    Insert,
    Delete,
    F(u8),
    Paste(String),
    Unknown(Vec<u8>),
}

const ESC: u8 = 0x1b;
const PASTE_END: &[u8] = b"\x1b[201~";

#[derive(Debug, Default)]
pub struct Parser {
    buf: Vec<u8>,
}

enum Parsed {
    /// キーと、消費したバイト数
    Key(Key, usize),
    /// まだバイトが足りない
    Incomplete,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { buf: Vec::new() }
    }

    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Key> {
        self.buf.extend_from_slice(bytes);
        let mut keys = Vec::new();
        let mut pos = 0;
        while pos < self.buf.len() {
            match parse_one(&self.buf[pos..]) {
                Parsed::Key(k, n) => {
                    keys.push(k);
                    pos += n;
                }
                Parsed::Incomplete => break,
            }
        }
        self.buf.drain(..pos);
        keys
    }

    pub fn flush(&mut self) -> Vec<Key> {
        if self.buf.is_empty() || self.buf.starts_with(b"\x1b[200~") {
            return Vec::new();
        }
        let pending = std::mem::take(&mut self.buf);
        let key = match pending.as_slice() {
            [ESC] => Key::Esc,
            [ESC, b'['] => Key::Alt('['),
            [ESC, b'O'] => Key::Alt('O'),
            _ => Key::Unknown(pending),
        };
        vec![key]
    }

    pub fn has_pending(&self) -> bool {
        !self.buf.is_empty()
    }
}

pub fn parse(bytes: &[u8]) -> Vec<Key> {
    let mut p = Parser::new();
    let mut keys = p.feed(bytes);
    keys.extend(p.flush());
    keys
}

fn parse_one(buf: &[u8]) -> Parsed {
    let b = buf[0];
    match b {
        ESC => parse_escape(buf),
        0x0d | 0x0a => Parsed::Key(Key::Enter, 1),
        0x09 => Parsed::Key(Key::Tab, 1),
        0x7f | 0x08 => Parsed::Key(Key::Backspace, 1),
        0x00 => Parsed::Key(Key::Ctrl(' '), 1),
        0x01..=0x1a => Parsed::Key(Key::Ctrl((b'a' + b - 1) as char), 1),
        0x1c => Parsed::Key(Key::Ctrl('\\'), 1),
        0x1d => Parsed::Key(Key::Ctrl(']'), 1),
        0x1e => Parsed::Key(Key::Ctrl('^'), 1),
        0x1f => Parsed::Key(Key::Ctrl('_'), 1),
        0x20..=0x7e => Parsed::Key(Key::Char(b as char), 1),
        _ => parse_utf8(buf),
    }
}

fn parse_utf8(buf: &[u8]) -> Parsed {
    let len = match buf[0] {
        0xc2..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf4 => 4,
        _ => return Parsed::Key(Key::Unknown(vec![buf[0]]), 1),
    };
    // 届いている分の継続バイトが正しいかを先に確認する
    for &c in buf.iter().take(len).skip(1) {
        if c & 0xc0 != 0x80 {
            return Parsed::Key(Key::Unknown(vec![buf[0]]), 1);
        }
    }
    if buf.len() < len {
        return Parsed::Incomplete;
    }
    match std::str::from_utf8(&buf[..len]) {
        Ok(s) => Parsed::Key(Key::Char(s.chars().next().unwrap()), len),
        Err(_) => Parsed::Key(Key::Unknown(vec![buf[0]]), 1),
    }
}

fn parse_escape(buf: &[u8]) -> Parsed {
    let Some(&next) = buf.get(1) else {
        return Parsed::Incomplete;
    };
    match next {
        b'[' => parse_csi(buf),
        b'O' => match buf.get(2) {
            None => Parsed::Incomplete,
            Some(&c) => {
                let key = match c {
                    b'A' => Key::Up,
                    b'B' => Key::Down,
                    b'C' => Key::Right,
                    b'D' => Key::Left,
                    b'H' => Key::Home,
                    b'F' => Key::End,
                    b'P' => Key::F(1),
                    b'Q' => Key::F(2),
                    b'R' => Key::F(3),
                    b'S' => Key::F(4),
                    _ => Key::Unknown(buf[..3].to_vec()),
                };
                Parsed::Key(key, 3)
            }
        },
        0x20..=0x7e => Parsed::Key(Key::Alt(next as char), 2),
        // ESC ESC や ESC + 制御文字/非 ASCII: ESC 単独として扱い、続きは次のキーとして読む
        _ => Parsed::Key(Key::Esc, 1),
    }
}

fn parse_csi(buf: &[u8]) -> Parsed {
    // buf = ESC [ params... final
    let mut i = 2;
    loop {
        let Some(&c) = buf.get(i) else {
            return Parsed::Incomplete;
        };
        match c {
            0x20..=0x3f => i += 1,
            0x40..=0x7e => break,
            _ => return Parsed::Key(Key::Unknown(buf[..i].to_vec()), i),
        }
    }
    let fin = buf[i];
    let len = i + 1;
    let params = std::str::from_utf8(&buf[2..i]).unwrap_or("");
    let first: Option<u32> = params.split(';').next().and_then(|s| s.parse().ok());
    let key = match fin {
        b'A' => Key::Up,
        b'B' => Key::Down,
        b'C' => Key::Right,
        b'D' => Key::Left,
        b'H' => Key::Home,
        b'F' => Key::End,
        b'Z' => Key::BackTab,
        b'P' => Key::F(1),
        b'Q' => Key::F(2),
        b'R' => Key::F(3),
        b'S' => Key::F(4),
        b'~' => match first {
            Some(1 | 7) => Key::Home,
            Some(2) => Key::Insert,
            Some(3) => Key::Delete,
            Some(4 | 8) => Key::End,
            Some(5) => Key::PageUp,
            Some(6) => Key::PageDown,
            Some(n @ 11..=15) => Key::F((n - 10) as u8),
            Some(n @ 17..=21) => Key::F((n - 11) as u8),
            Some(n @ 23..=24) => Key::F((n - 12) as u8),
            Some(200) => return parse_paste(buf, len),
            _ => Key::Unknown(buf[..len].to_vec()),
        },
        _ => Key::Unknown(buf[..len].to_vec()),
    };
    Parsed::Key(key, len)
}

fn parse_paste(buf: &[u8], start: usize) -> Parsed {
    let body = &buf[start..];
    match body.windows(PASTE_END.len()).position(|w| w == PASTE_END) {
        None => Parsed::Incomplete,
        Some(p) => {
            let text = String::from_utf8_lossy(&body[..p]).into_owned();
            Parsed::Key(Key::Paste(text), start + p + PASTE_END.len())
        }
    }
}
