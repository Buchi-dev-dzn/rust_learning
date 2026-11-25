スレッドを起動する関数はどれも同じ形です。`thread::spawn` の戻り値 (`JoinHandle`) をそのまま返します。

```rust
thread::spawn(move || {
    for k in keys {
        if tx.send(Event::Key(k)).is_err() {
            return; // 受信側が消えていたら終わる
        }
    }
})
```

`move` がないと、`tx` と `keys` をスレッドに渡せません。

---

`App::handle` は `match` で。Backspace は `'q'` と同じく具体的な文字のパターンとして、
汎用の `Event::Key(c)` より **前** に書きます。`String::pop()` は空でもパニックしません。

```rust
match event {
    Event::Key('q') => self.quit = true,
    Event::Key('\u{8}') => { self.text.pop(); }
    Event::Key(c) => self.text.push(c),
    Event::Tick => self.ticks += 1,
}
```

---

`run` のループ:

```rust
while !app.quit {
    match rx.recv() {
        Ok(ev) => app.handle(ev),
        Err(_) => break,
    }
}
```

`spawn_ticker` は `loop { thread::sleep(interval); if tx.send(Event::Tick).is_err() { return; } }`。
`run` が `rx` を値で受け取っているので、`run` から戻った時点で `rx` が drop され、ticker の `send` が失敗して終了します。
