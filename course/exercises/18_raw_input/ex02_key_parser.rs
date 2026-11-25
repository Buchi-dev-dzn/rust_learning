//! # キー入力パーサ (状態機械)
//!
//! stdin から届くバイトを 1 つずつ `feed` して、キーが確定したら `Some(Key)` を返す
//! 状態機械 `KeyParser` を完成させよう。
//!
//! 状態と遷移:
//!
//! - `Ground` (通常)
//!   - `0x1B` → `Escape` へ (まだ確定しない)
//!   - `0x0D` → `Enter` / `0x09` → `Tab` / `0x7F`・`0x08` → `Backspace`
//!   - `0x01..=0x1A` → `Ctrl(小文字)` / `0x20..=0x7E` → `Char`
//!   - UTF-8 の先頭バイト `0xC0..=0xDF` (続き 1 バイト)・`0xE0..=0xEF` (続き 2)・`0xF0..=0xF7` (続き 3)
//!     → `Utf8` へ。続きが全部そろったら `Char(その文字)` (不正なら `Unknown(そのバイト列)`)
//!   - それ以外 → `Unknown(vec![b])`
//! - `Escape` (ESC を 1 つ読んだ)
//!   - `[` → `Csi` へ / `O` → `Ss3` へ
//!   - `0x1B` → `Esc` を返し、`Escape` のまま (ESC ESC = Esc が 1 回押されて次の ESC が来た)
//!   - `0x20..=0x7E` → `Alt(文字)`
//!   - それ以外 → `Unknown(vec![0x1B, b])`
//! - `Csi` (`ESC [` の後)
//!   - `0x30..=0x3F` (数字・`;` など) → 引数として貯める
//!   - `0x40..=0x7E` (終端) → キーを確定:
//!     - `A`/`B`/`C`/`D` → `Up`/`Down`/`Right`/`Left`、`H` → `Home`、`F` → `End`
//!       (`ESC [ 1 ; 5 C` のような修飾付きも引数を無視して同じキーにする)
//!     - `~` → 最初の引数 (`;` より前) で決める: `1`・`7` → `Home`, `2` → `Insert`, `3` → `Delete`,
//!       `4`・`8` → `End`, `5` → `PageUp`, `6` → `PageDown`, `15` → `F(5)`, `17`〜`21` → `F(6)`〜`F(10)`,
//!       `23` → `F(11)`, `24` → `F(12)`
//!     - それ以外 → `Unknown(ESC [ からの全バイト)`
//!   - その他のバイト → `Unknown(ESC [ からの全バイト)`
//! - `Ss3` (`ESC O` の後)
//!   - `P`/`Q`/`R`/`S` → `F(1)`〜`F(4)`、`A`〜`D` → 矢印、`H` → `Home`、`F` → `End`
//!   - それ以外 → `Unknown(vec![0x1B, b'O', b])`
//!
//! キーが確定したら状態は `Ground` に戻る。
//!
//! `flush()` は「しばらく入力が来なかった」ときに呼ぶ。`Escape` 状態なら `Some(Esc)`、
//! それ以外の途中状態なら `Some(Unknown(途中までのバイト列))`、`Ground` なら `None`。
//! どの場合も `Ground` に戻る。
//!
//! - 関連: 第18章「Esc キー問題」「キーパーサを状態機械で書く」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, PartialEq, Clone)]
enum Key {
    Char(char),
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
    Insert,
    Delete,
    PageUp,
    PageDown,
    F(u8),
    Unknown(Vec<u8>),
}

#[derive(Debug)]
enum State {
    Ground,
    Escape,
    Csi(Vec<u8>),                        // ESC [ の後に貯めた引数バイト
    Ss3,
    Utf8 { buf: Vec<u8>, need: usize }, // need = 残り何バイト必要か
}

struct KeyParser {
    state: State,
}

impl KeyParser {
    fn new() -> Self {
        KeyParser { state: State::Ground }
    }

    // TODO: 現在の状態とバイト b から、次の状態と (確定したなら) キーを決める
    // ヒント: `let state = std::mem::replace(&mut self.state, State::Ground);` で
    //         状態を取り出してから match すると、Vec を所有権ごと扱える
    fn feed(&mut self, b: u8) -> Option<Key> {
        todo!()
    }

    // TODO: 途中状態を確定させて Ground に戻す
    fn flush(&mut self) -> Option<Key> {
        todo!()
    }
}

/// バイト列を全部食べさせ、最後に flush する (1 回の read で届いた分を解析するイメージ)
fn parse_all(bytes: &[u8]) -> Vec<Key> {
    let mut p = KeyParser::new();
    let mut keys: Vec<Key> = bytes.iter().filter_map(|&b| p.feed(b)).collect();
    keys.extend(p.flush());
    keys
}

fn main() {
    let input = "aあ\x1b[A\x1b[3~\x1bOP\x1bx\x1b".as_bytes();
    for k in parse_all(input) {
        println!("{k:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_bytes() {
        assert_eq!(
            parse_all(b"a\r\t\x7f\x03"),
            vec![Key::Char('a'), Key::Enter, Key::Tab, Key::Backspace, Key::Ctrl('c')]
        );
    }

    #[test]
    fn feed_returns_none_until_complete() {
        let mut p = KeyParser::new();
        assert_eq!(p.feed(0x1b), None);
        assert_eq!(p.feed(b'['), None);
        assert_eq!(p.feed(b'A'), Some(Key::Up));
        assert_eq!(p.feed(b'x'), Some(Key::Char('x')));
    }

    #[test]
    fn arrows_home_end() {
        assert_eq!(
            parse_all(b"\x1b[A\x1b[B\x1b[C\x1b[D\x1b[H\x1b[F"),
            vec![Key::Up, Key::Down, Key::Right, Key::Left, Key::Home, Key::End]
        );
        // 修飾付き (Ctrl+→) は引数を無視
        assert_eq!(parse_all(b"\x1b[1;5C"), vec![Key::Right]);
    }

    #[test]
    fn tilde_keys() {
        assert_eq!(
            parse_all(b"\x1b[1~\x1b[2~\x1b[3~\x1b[4~\x1b[5~\x1b[6~\x1b[7~\x1b[8~"),
            vec![
                Key::Home, Key::Insert, Key::Delete, Key::End,
                Key::PageUp, Key::PageDown, Key::Home, Key::End
            ]
        );
        assert_eq!(
            parse_all(b"\x1b[15~\x1b[17~\x1b[21~\x1b[23~\x1b[24~\x1b[3;5~"),
            vec![Key::F(5), Key::F(6), Key::F(10), Key::F(11), Key::F(12), Key::Delete]
        );
    }

    #[test]
    fn ss3_keys() {
        assert_eq!(
            parse_all(b"\x1bOP\x1bOQ\x1bOR\x1bOS\x1bOA\x1bOH"),
            vec![Key::F(1), Key::F(2), Key::F(3), Key::F(4), Key::Up, Key::Home]
        );
    }

    #[test]
    fn alt_and_esc() {
        assert_eq!(parse_all(b"\x1bx"), vec![Key::Alt('x')]);
        assert_eq!(parse_all(b"\x1b"), vec![Key::Esc]);
        assert_eq!(parse_all(b"\x1b\x1b"), vec![Key::Esc, Key::Esc]);
        assert_eq!(parse_all(b"\x1b\x1b[A"), vec![Key::Esc, Key::Up]);
    }

    #[test]
    fn utf8() {
        assert_eq!(
            parse_all("é日🦀".as_bytes()),
            vec![Key::Char('é'), Key::Char('日'), Key::Char('🦀')]
        );
        let mut p = KeyParser::new();
        assert_eq!(p.feed(0xe6), None);
        assert_eq!(p.feed(0x97), None);
        assert_eq!(p.feed(0xa5), Some(Key::Char('日')));
    }

    #[test]
    fn unknown_sequences() {
        assert_eq!(parse_all(b"\x1b[99~"), vec![Key::Unknown(b"\x1b[99~".to_vec())]);
        assert_eq!(parse_all(b"\x1b[Z"), vec![Key::Unknown(b"\x1b[Z".to_vec())]);
        assert_eq!(parse_all(b"\x1bOx"), vec![Key::Unknown(b"\x1bOx".to_vec())]);
        assert_eq!(parse_all(&[0x1b, 0x01]), vec![Key::Unknown(vec![0x1b, 0x01])]);
        assert_eq!(parse_all(&[0xe6, b'a']), vec![Key::Unknown(vec![0xe6, b'a'])]);
        assert_eq!(parse_all(&[0xff]), vec![Key::Unknown(vec![0xff])]);
    }

    #[test]
    fn flush_incomplete() {
        assert_eq!(parse_all(b"\x1b[1;"), vec![Key::Unknown(b"\x1b[1;".to_vec())]);
        assert_eq!(parse_all(&[0xe6, 0x97]), vec![Key::Unknown(vec![0xe6, 0x97])]);
        let mut p = KeyParser::new();
        assert_eq!(p.flush(), None);
        p.feed(0x1b);
        assert_eq!(p.flush(), Some(Key::Esc));
        assert_eq!(p.feed(b'['), Some(Key::Char('['))); // flush 後は Ground
    }
}
