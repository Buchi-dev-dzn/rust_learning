外側をモードで、内側を Msg で分ける 2 段の `match` にすると整理できます。

```rust
match &mut self.mode {
    Mode::Normal => match msg { /* ... */ },
    Mode::Adding { input } => match msg { /* ... */ },
}
false
```

`&mut self.mode` で match すると、`Adding { input }` の `input` が `&mut String` として取り出せ、
そのまま `input.push(c)` できます。`self.items` は別フィールドなので同時に触っても借用エラーになりません。

---

範囲を守る計算:

- Up: `self.selected = self.selected.saturating_sub(1);` (0 から引いても 0)
- Down: `if self.selected + 1 < self.items.len() { self.selected += 1; }`
- Delete 後: `self.selected = self.selected.min(self.items.len().saturating_sub(1));`
- Toggle: `if let Some(item) = self.items.get_mut(self.selected) { item.done = !item.done; }` (空でも安全)

---

Submit の処理:

```rust
Msg::Submit => {
    let title = input.trim().to_string();
    if !title.is_empty() {
        self.items.push(Item { title, done: false });
        self.selected = self.items.len() - 1;
    }
    self.mode = Mode::Normal;
}
```

`view` は `self.items.iter().enumerate().map(|(i, item)| format!("{cursor}{mark}{}", item.title))` で行を作り、
空なら `"  (項目なし)"` を、`Adding` なら `format!("追加: {input}_")` を最後に push します。
