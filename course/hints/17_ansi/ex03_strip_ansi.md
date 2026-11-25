`let mut chars = s.chars();` とイテレータを変数に入れ、`while let Some(c) = chars.next()` で回します。
`for` ではなくこの形にすると、ループの中でも `chars.next()` を呼んで先読み・読み飛ばしができます。

---

`c == '\x1b'` のとき、次の 1 文字を `chars.next()` で取り出して分岐します。

- `Some('[')` → 終端バイト (`'\x40'..='\x7e'` の範囲) が来るまで `chars.next()` を繰り返す
- `Some(_)` → その 1 文字ごと捨てる
- `None` → 何もしない (末尾)

`[` 自体は `0x5B` で終端バイトの範囲に入るので、`[` を読んだ後から終端を探す点に注意。

---

```rust
let mut out = String::new();
let mut chars = s.chars();
while let Some(c) = chars.next() {
    if c != '\x1b' { out.push(c); continue; }
    if let Some('[') = chars.next() {
        while let Some(b) = chars.next() {
            if ('\x40'..='\x7e').contains(&b) { break; }
        }
    }
}
out
```
