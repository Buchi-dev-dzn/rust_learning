`inverse` は 8 通りの `match` です。`use Command::*;` を関数の中に書くと `Command::` を省略できます。

`new` では実行するコマンドの列 `steps` を `Vec` で作り (mouse なら `EnableMouse` を追加)、
順番に `backend.execute(cmd)` します。成功したものは `done` に積んでおきます。

---

ロールバックは「`done` を逆順に、`inverse` を実行」です。

```rust
for cmd in steps {
    if let Err(e) = backend.execute(cmd) {
        for &d in done.iter().rev() {
            let _ = backend.execute(inverse(d));
        }
        return Err(e);
    }
    done.push(cmd);
}
Ok(TerminalGuard { backend, done })
```

`restore` も同じ「逆順に打ち消す」処理です。エラーは `let _ =` で無視して続行します。

---

冪等にするには、`restore` の最初で `done` を空にしてしまうのが簡単です。

```rust
fn restore(&mut self) {
    let done = std::mem::take(&mut self.done); // self.done は空の Vec になる
    for &d in done.iter().rev() {
        let _ = self.backend.execute(inverse(d));
    }
}
```

2 回目は `done` が空なので何も実行されません。`Drop` では `self.restore();` を呼びます。
