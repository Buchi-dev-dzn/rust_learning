関数の引数にも `mut` を付けられます: `fn pad_right(mut s: String, width: usize)`。
受け取った `String` は自分のものなので、自由に書き換えてから返せます。

---

- `pad_right`: `s.len()` はバイト数です。日本語を含むので `s.chars().count()` で文字数を数え、足りない分だけ `s.push(' ')`。
- `join_status`: `format!("{left} | {right}")` が一番簡単です。
- `submit`: `std::mem::take(input)` は `input` を空の `String` と入れ替え、元の中身を返します。

---

```rust
fn pad_right(mut s: String, width: usize) -> String {
    let len = s.chars().count();
    for _ in len..width {
        s.push(' ');
    }
    s
}

fn submit(input: &mut String) -> String {
    std::mem::take(input)
}
```
