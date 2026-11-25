`match` の腕は上から順に試されます。9 (Tab) と 13 (Enter) は範囲 `1..=26` にも含まれるので、
範囲の腕より **上** に書きましょう。

---

- `key_from_byte`: `match b { 9 => Some(Key::Tab), 13 => ..., 27 => ..., 127 => ..., 1..=26 => Some(Key::Ctrl((b'a' + b - 1) as char)), 32..=126 => Some(Key::Char(b as char)), _ => None }`
- `action_for`: `Key::Ctrl('q') | Key::Esc => Action::Quit` のように `|` でまとめられます。`Key::Ctrl('s')` を `Key::Ctrl(_)` より先に。

---

```rust
fn action_for(key: Key) -> Action {
    match key {
        Key::Ctrl('q') | Key::Esc => Action::Quit,
        Key::Ctrl('s') => Action::Save,
        Key::Ctrl(_) => Action::Nothing,
        Key::Char(c) => Action::Insert(c),
        Key::Tab => Action::Insert(' '),
        Key::Backspace => Action::DeleteBack,
        Key::Enter => Action::NewLine,
    }
}
```
