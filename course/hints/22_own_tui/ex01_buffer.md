`new` は `vec![Cell { ch: ' ' }; count]` を使えます。`Cell` に `Clone` が必要です。

---

`put` は `x < width && y < height` を確認してから `cells[y * width + x]` に書きます。先に添字を計算しないでください。

---

`draw_text` は範囲外ならすぐ戻り、`text.chars().take(self.width - x).enumerate()` で `put(x + offset, y, ch)` します。幅と高さの積は `checked_mul` で検査できます。
