`update` は `match msg { Msg::Increment => ..., ... }` で全バリアントを処理します。
範囲に収めるには `i64::clamp` が使えます: `x.clamp(-1000, 1000)`。

---

「count を変える」処理が `Increment` / `Decrement` / `Reset` の 3 か所で同じなので、補助関数にまとめます。

```rust
fn set_count(m: &mut Model, new: i64) {
    let new = new.clamp(-1000, 1000);
    if new != m.count {
        m.history.push(m.count); // 変更前の値
        m.count = new;
    }
}
```

---

```rust
match msg {
    Msg::Increment => set_count(m, m.count + m.step),
    Msg::Decrement => set_count(m, m.count - m.step),
    Msg::SetStep(n) => m.step = n.clamp(1, 100),
    Msg::Reset => set_count(m, 0),
    Msg::Undo => {
        if let Some(prev) = m.history.pop() { m.count = prev; }
    }
    Msg::Quit => m.quit = true,
}
```

`view` は `format!("count: {} (step {}) undo:{}", m.count, m.step, m.history.len())` です。
