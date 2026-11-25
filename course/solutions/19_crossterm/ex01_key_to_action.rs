//! # キーイベントをアクションに変換する
//!
//! crossterm の `KeyEvent` を真似た型 (標準ライブラリのみ) を使って、
//! vim 風エディタのキー割り当て `map_key` を実装しよう。
//! 本番では引数が `crossterm::event::KeyEvent` になるだけで、考え方は同じ。
//!
//! 共通ルール:
//! - `kind` が `Press` 以外 (`Release`, `Repeat`) … `Repeat` は `Press` と同じに扱い、`Release` は `None`
//! - Ctrl+C (`Char('c')` + ctrl) … どのモードでも `Quit`
//!
//! `Mode::Normal` のとき:
//! - `q` → `Quit`、`i` → `EnterInsert`
//! - `h`/`j`/`k`/`l` または ←/↓/↑/→ → `Move(dx, dy)` (h=(-1,0), j=(0,1), k=(0,-1), l=(1,0))
//! - Shift 付きの ↓ / ↑ (または `J` / `K`) → `Move(0, 10)` / `Move(0, -10)` (10 行ずつ)
//! - Ctrl+S → `Save`
//! - それ以外 → `None`
//!
//! `Mode::Insert` のとき:
//! - `Esc` → `LeaveInsert`
//! - Ctrl なしの `Char(c)` → `Insert(c)` (Shift は関係ない。大文字もそのまま)
//! - `Enter` → `Insert('\n')`、`Backspace` → `DeleteBack`
//! - 矢印キー → `Move` (Normal と同じ。Shift で 10 行も同じ)
//! - Ctrl+S → `Save`
//! - それ以外 → `None`
//!
//! - 関連: 第19章「イベントを読む」「テストしやすい設計」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyCode {
    Char(char),
    Enter,
    Backspace,
    Esc,
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Modifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyEventKind {
    Press,
    Repeat,
    Release,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct KeyEvent {
    code: KeyCode,
    modifiers: Modifiers,
    kind: KeyEventKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Mode {
    Normal,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Action {
    Quit,
    Save,
    EnterInsert,
    LeaveInsert,
    Move(i32, i32),
    Insert(char),
    DeleteBack,
}

/// 矢印キー (両モード共通)。Shift なら縦は 10 行
fn arrow(code: KeyCode, shift: bool) -> Option<Action> {
    let step = if shift { 10 } else { 1 };
    match code {
        KeyCode::Left => Some(Action::Move(-1, 0)),
        KeyCode::Right => Some(Action::Move(1, 0)),
        KeyCode::Up => Some(Action::Move(0, -step)),
        KeyCode::Down => Some(Action::Move(0, step)),
        _ => None,
    }
}

fn map_key(ev: KeyEvent, mode: Mode) -> Option<Action> {
    if ev.kind == KeyEventKind::Release {
        return None;
    }
    let m = ev.modifiers;
    // 両モード共通の Ctrl キー
    if m.ctrl {
        return match ev.code {
            KeyCode::Char('c') => Some(Action::Quit),
            KeyCode::Char('s') => Some(Action::Save),
            _ => None,
        };
    }
    if let Some(a) = arrow(ev.code, m.shift) {
        return Some(a);
    }
    match mode {
        Mode::Normal => match ev.code {
            KeyCode::Char('q') => Some(Action::Quit),
            KeyCode::Char('i') => Some(Action::EnterInsert),
            KeyCode::Char('h') => Some(Action::Move(-1, 0)),
            KeyCode::Char('j') => Some(Action::Move(0, 1)),
            KeyCode::Char('k') => Some(Action::Move(0, -1)),
            KeyCode::Char('l') => Some(Action::Move(1, 0)),
            KeyCode::Char('J') => Some(Action::Move(0, 10)),
            KeyCode::Char('K') => Some(Action::Move(0, -10)),
            _ => None,
        },
        Mode::Insert => match ev.code {
            KeyCode::Esc => Some(Action::LeaveInsert),
            KeyCode::Char(c) => Some(Action::Insert(c)),
            KeyCode::Enter => Some(Action::Insert('\n')),
            KeyCode::Backspace => Some(Action::DeleteBack),
            _ => None,
        },
    }
}

fn main() {
    let ev = KeyEvent { code: KeyCode::Char('j'), modifiers: Modifiers::default(), kind: KeyEventKind::Press };
    println!("{:?}", map_key(ev, Mode::Normal));
    println!("{:?}", map_key(ev, Mode::Insert));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent { code, modifiers: Modifiers::default(), kind: KeyEventKind::Press }
    }
    fn ctrl(c: char) -> KeyEvent {
        KeyEvent {
            code: KeyCode::Char(c),
            modifiers: Modifiers { ctrl: true, ..Default::default() },
            kind: KeyEventKind::Press,
        }
    }
    fn shift(code: KeyCode) -> KeyEvent {
        KeyEvent { code, modifiers: Modifiers { shift: true, ..Default::default() }, kind: KeyEventKind::Press }
    }
    use KeyCode::*;

    #[test]
    fn release_is_ignored_repeat_is_press() {
        let mut ev = key(Char('q'));
        ev.kind = KeyEventKind::Release;
        assert_eq!(map_key(ev, Mode::Normal), None);
        ev.kind = KeyEventKind::Repeat;
        assert_eq!(map_key(ev, Mode::Normal), Some(Action::Quit));
    }

    #[test]
    fn ctrl_c_quits_everywhere() {
        assert_eq!(map_key(ctrl('c'), Mode::Normal), Some(Action::Quit));
        assert_eq!(map_key(ctrl('c'), Mode::Insert), Some(Action::Quit));
        assert_eq!(map_key(ctrl('s'), Mode::Normal), Some(Action::Save));
        assert_eq!(map_key(ctrl('s'), Mode::Insert), Some(Action::Save));
        assert_eq!(map_key(ctrl('x'), Mode::Insert), None);
    }

    #[test]
    fn normal_mode() {
        assert_eq!(map_key(key(Char('q')), Mode::Normal), Some(Action::Quit));
        assert_eq!(map_key(key(Char('i')), Mode::Normal), Some(Action::EnterInsert));
        assert_eq!(map_key(key(Char('h')), Mode::Normal), Some(Action::Move(-1, 0)));
        assert_eq!(map_key(key(Char('j')), Mode::Normal), Some(Action::Move(0, 1)));
        assert_eq!(map_key(key(Char('k')), Mode::Normal), Some(Action::Move(0, -1)));
        assert_eq!(map_key(key(Char('l')), Mode::Normal), Some(Action::Move(1, 0)));
        assert_eq!(map_key(key(Char('x')), Mode::Normal), None);
        assert_eq!(map_key(key(Esc), Mode::Normal), None);
        assert_eq!(map_key(key(Enter), Mode::Normal), None);
    }

    #[test]
    fn arrows_and_shift() {
        for mode in [Mode::Normal, Mode::Insert] {
            assert_eq!(map_key(key(Left), mode), Some(Action::Move(-1, 0)));
            assert_eq!(map_key(key(Right), mode), Some(Action::Move(1, 0)));
            assert_eq!(map_key(key(Up), mode), Some(Action::Move(0, -1)));
            assert_eq!(map_key(key(Down), mode), Some(Action::Move(0, 1)));
            assert_eq!(map_key(shift(Down), mode), Some(Action::Move(0, 10)));
            assert_eq!(map_key(shift(Up), mode), Some(Action::Move(0, -10)));
        }
        assert_eq!(map_key(shift(Char('J')), Mode::Normal), Some(Action::Move(0, 10)));
        assert_eq!(map_key(shift(Char('K')), Mode::Normal), Some(Action::Move(0, -10)));
    }

    #[test]
    fn insert_mode() {
        assert_eq!(map_key(key(Esc), Mode::Insert), Some(Action::LeaveInsert));
        assert_eq!(map_key(key(Char('q')), Mode::Insert), Some(Action::Insert('q')));
        assert_eq!(map_key(key(Char('あ')), Mode::Insert), Some(Action::Insert('あ')));
        assert_eq!(map_key(shift(Char('A')), Mode::Insert), Some(Action::Insert('A')));
        assert_eq!(map_key(key(Enter), Mode::Insert), Some(Action::Insert('\n')));
        assert_eq!(map_key(key(Backspace), Mode::Insert), Some(Action::DeleteBack));
    }
}
