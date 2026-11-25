`make_counter` は、ローカル変数 `count` を **所有したまま** 返る必要があります。
関数を抜けると `count` は消えるので、`move` を付けてクロージャの中に移動させます。
クロージャの本体で `count += 1; count` とすれば `FnMut` になります。

`make_clamper` も同様に `move |v| ...`。`i64::clamp(v, 0, max)` が使えます。

---

`Keymap`:

```rust
pub fn bind(&mut self, key: char, action: impl Fn(&mut App) + 'static) {
    self.actions.insert(key, Box::new(action));
}
```

`handle` は `match self.actions.get(&key)` で `Some(action)` なら `action(app); true`。
`action` は `&Box<dyn Fn(&mut App)>` ですが、そのまま関数のように呼べます。

---

```rust
km.bind('j', |app| {
    if app.cursor + 1 < app.len {
        app.cursor += 1;
    }
});
km.bind('k', |app| app.cursor = app.cursor.saturating_sub(1));
km.bind('q', |app| app.quit = true);
```
