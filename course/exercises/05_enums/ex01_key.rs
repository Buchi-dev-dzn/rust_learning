//! # Key enum とキー解析
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: データを持つ enum を `match` で網羅的に処理し、`Option` を返す
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第5章「データを持つ enum」「match」「Option」
//!
//! ターミナルを raw モードにすると、キー入力は 1 バイトずつ届く (第18章)。
//! ここではその簡易版として、1 バイトを `Key` に変換する関数と、`Key` をアプリの操作に
//! 変換する関数を書く。

#[derive(Debug, Clone, Copy, PartialEq)]
enum Key {
    Char(char),
    Ctrl(char), // Ctrl + 英小文字
    Enter,
    Esc,
    Backspace,
    Tab,
}

#[derive(Debug, PartialEq)]
enum Action {
    Quit,
    Save,
    Insert(char),
    DeleteBack,
    NewLine,
    Nothing,
}

/// 1 バイトを Key に変換する。対応しないバイトは None。
///
/// - 9 (`\t`) → Tab
/// - 13 (`\r`) → Enter
/// - 27 → Esc
/// - 127 → Backspace
/// - 1..=26 (9 と 13 を除く) → Ctrl('a'..='z')  (1 が 'a'、26 が 'z')
/// - 32..=126 (表示可能な ASCII) → Char(その文字)
/// - それ以外 → None
fn key_from_byte(b: u8) -> Option<Key> {
    // TODO: match b { 9 => ..., 1..=26 => ..., _ => None } のように範囲パターンを使う
    //       数値から文字へは (b'a' + b - 1) as char のように変換できる
    todo!()
}

/// Key をアプリの操作に変換する。
///
/// - Ctrl('q') または Esc → Quit
/// - Ctrl('s') → Save
/// - その他の Ctrl → Nothing
/// - Char(c) → Insert(c)
/// - Tab → Insert(' ')  (タブは空白 1 つとして入力)
/// - Backspace → DeleteBack
/// - Enter → NewLine
fn action_for(key: Key) -> Action {
    // TODO: `_` を使わずに全バリアントを書いてみよう (網羅性チェックを体験する)
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bytes() {
        assert_eq!(key_from_byte(b'a'), Some(Key::Char('a')));
        assert_eq!(key_from_byte(b' '), Some(Key::Char(' ')));
        assert_eq!(key_from_byte(b'~'), Some(Key::Char('~')));
        assert_eq!(key_from_byte(13), Some(Key::Enter));
        assert_eq!(key_from_byte(9), Some(Key::Tab));
        assert_eq!(key_from_byte(27), Some(Key::Esc));
        assert_eq!(key_from_byte(127), Some(Key::Backspace));
        assert_eq!(key_from_byte(1), Some(Key::Ctrl('a')));
        assert_eq!(key_from_byte(17), Some(Key::Ctrl('q')));
        assert_eq!(key_from_byte(26), Some(Key::Ctrl('z')));
        assert_eq!(key_from_byte(0), None);
        assert_eq!(key_from_byte(200), None);
    }

    #[test]
    fn maps_actions() {
        assert_eq!(action_for(Key::Ctrl('q')), Action::Quit);
        assert_eq!(action_for(Key::Esc), Action::Quit);
        assert_eq!(action_for(Key::Ctrl('s')), Action::Save);
        assert_eq!(action_for(Key::Ctrl('x')), Action::Nothing);
        assert_eq!(action_for(Key::Char('z')), Action::Insert('z'));
        assert_eq!(action_for(Key::Tab), Action::Insert(' '));
        assert_eq!(action_for(Key::Backspace), Action::DeleteBack);
        assert_eq!(action_for(Key::Enter), Action::NewLine);
    }

    #[test]
    fn bytes_to_actions() {
        let input = b"hi\x7f\x13\x11";
        let actions: Vec<Action> = input
            .iter()
            .filter_map(|&b| key_from_byte(b))
            .map(action_for)
            .collect();
        assert_eq!(
            actions,
            vec![Action::Insert('h'), Action::Insert('i'), Action::DeleteBack, Action::Save, Action::Quit]
        );
    }
}
