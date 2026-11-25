`Tick` では「変更前の表示単位」と「変更後の表示単位」を比べます。

```rust
let before = self.elapsed_ms / 100;
self.elapsed_ms += ms;
self.elapsed_ms / 100 != before
```

停止中なら最初に `return false;` です。

---

`view` では 0.1 秒単位の値 `t = elapsed_ms / 100` から分・秒・0.1 秒を求めます。

- 分: `t / 600`
- 秒: `t / 10 % 60`
- 0.1 秒: `t % 10`

`format!("{min:02}:{sec:02}.{d} {icon}")` の `:02` は「2 桁でゼロ埋め」です。

---

```rust
fn run(events: impl IntoIterator<Item = Event>) -> Vec<String> {
    let mut app = Stopwatch::default();
    let mut frames = vec![app.view()];
    for ev in events {
        let dirty = app.update(ev);
        if app.quit { break; }
        if dirty { frames.push(app.view()); }
    }
    frames
}
```

`Key('r')` は「変わったか」を先に計算してから値を書き換えます: `let changed = self.running || self.elapsed_ms != 0;`
