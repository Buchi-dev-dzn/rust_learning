`binary operation == cannot be applied to type Rgb` と `Rgb doesn't implement Debug` は、
どちらも derive を付ければ解決します。

---

- `Rgb` に `#[derive(Debug, PartialEq)]` を付けます。
- `name(self)` は呼んだ時点で `t` をムーブしてしまうので、2 回目の呼び出しができません。`&self` にしましょう。戻り値も `String` のままだと `self.name` をムーブできない (E0507) ので、`&str` を返すか `self.name.clone()` を返します。
- `invert` は自分を書き換えるので `&mut self`。

---

```rust
#[derive(Debug, PartialEq)]
struct Rgb(u8, u8, u8);

fn name(&self) -> &str {
    &self.name
}

fn invert(&mut self) {
    std::mem::swap(&mut self.fg, &mut self.bg);
}
```
