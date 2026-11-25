まず `let cp = c as u32;` で数値にします。幅 0 の条件 (制御文字・結合文字) を先に判定して `return 0;` し、
その後で幅 2 の範囲を調べると整理しやすいです。

---

範囲の判定は `(0x0300..=0x036F).contains(&cp)` か、`matches!` マクロで書けます。

```rust
let wide = matches!(cp,
    0x1100..=0x115F
    | 0x2E80..=0x303E
    // ...
);
```

半角カナ (U+FF61〜) を 2 にしないよう、全角英数の範囲は `0xFF00..=0xFF60` で止めます。

---

```rust
if cp < 0x20 || (0x7F..=0x9F).contains(&cp) { return 0; }
if (0x0300..=0x036F).contains(&cp) || (0x200B..=0x200F).contains(&cp) || cp == 0xFE0F { return 0; }
let wide = matches!(cp, 0x1100..=0x115F | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x3400..=0x4DBF
    | 0x4E00..=0x9FFF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6
    | 0x1F300..=0x1F64F | 0x1F900..=0x1F9FF | 0x20000..=0x3FFFD);
if wide { 2 } else { 1 }
```

`str_width` は `s.chars().map(char_width).sum()` です。
