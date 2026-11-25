//! F3: キー入力パーサ。
//!
//! 仕様は SPEC.md。`todo!()` を埋めて `cargo test -p f3_keyparser` を通そう。
//! `Key` の定義と公開関数のシグネチャはテストが依存しているので変えないこと。
//! (Parser のフィールドや補助関数は自由に変えてよい)

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

#[derive(Debug, Default)]
pub struct Parser {
    /// まだキーに変換していないバイト
    buf: Vec<u8>,
}

impl Parser {
    pub fn new() -> Parser {
        Parser { buf: Vec::new() }
    }

    /// バイトを追加し、完成したキーをすべて返す。未完成の部分は保持する。
    pub fn feed(&mut self, bytes: &[u8]) -> Vec<Key> {
        // TODO: self.buf に追記 → 先頭から 1 キーずつ切り出す
        todo!()
    }

    /// タイムアウト時に呼ぶ。保持中のバイトを確定させる (SPEC 3章)。
    pub fn flush(&mut self) -> Vec<Key> {
        // TODO
        todo!()
    }

    pub fn has_pending(&self) -> bool {
        // TODO
        todo!()
    }
}

/// 新しい Parser で feed してから flush した結果。
pub fn parse(bytes: &[u8]) -> Vec<Key> {
    // TODO
    todo!()
}
