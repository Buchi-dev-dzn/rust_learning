いきなり大きな `match` を書くより、ルールを順番に「早期 return」で処理すると整理できます。

1. `if ev.kind == KeyEventKind::Release { return None; }`
2. Ctrl が押されていたら `c` → `Quit`、`s` → `Save`、それ以外 `None` で return
3. 矢印キーなら `Move` を return (両モード共通)
4. `match mode { Mode::Normal => ..., Mode::Insert => ... }`

---

矢印キーの処理は補助関数にすると Normal / Insert の両方で使い回せます。

```rust
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
```

`if let Some(a) = arrow(ev.code, ev.modifiers.shift) { return Some(a); }` のように使います。

---

Insert モードは次のようになります。`KeyCode::Char(c)` は Ctrl を手順 2 で処理済みなので、そのまま `Insert(c)` にできます。

```rust
Mode::Insert => match ev.code {
    KeyCode::Esc => Some(Action::LeaveInsert),
    KeyCode::Char(c) => Some(Action::Insert(c)),
    KeyCode::Enter => Some(Action::Insert('\n')),
    KeyCode::Backspace => Some(Action::DeleteBack),
    _ => None,
},
```

Normal モードは `'q' 'i' 'h' 'j' 'k' 'l' 'J' 'K'` をそれぞれ書き、残りは `_ => None` です。
