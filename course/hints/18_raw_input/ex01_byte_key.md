`match b { 0x0d => Key::Enter, ... }` のように書けます。範囲は `0x01..=0x1a` の形でパターンにできます。
`0x0d` (Enter) や `0x09` (Tab) も `0x01..=0x1a` の範囲に入るので、**特別なキーを先に** 書きましょう。

---

Ctrl の文字は `0x01` が `'a'` なので、`(b - 1 + b'a') as char` で求まります。
`0x20..=0x7e` は `Key::Char(b as char)` です。

`ctrl_byte` は `c.is_ascii_alphabetic()` で英字か確かめ、`c as u8 & 0x1f` を返します。

---

```rust
match b {
    0x0d => Key::Enter,
    0x09 => Key::Tab,
    0x7f | 0x08 => Key::Backspace,
    0x1b => Key::Esc,
    0x01..=0x1a => Key::Ctrl((b - 1 + b'a') as char),
    0x20..=0x7e => Key::Char(b as char),
    _ => Key::Unknown(b),
}
```
