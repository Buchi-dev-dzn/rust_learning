空のリストは例外です。先に長さを確認し、`None` を維持しましょう。`Replace` の後にも空かどうかを調べます。

---

`Next` は `Some(i)` で `i < len - 1` なら `i + 1`、それ以外は `0`。`Previous` は `Some(i)` で `i > 0` なら `i - 1`、それ以外は `len - 1`。

---

`Replace` 後は `selected.map(|i| i.min(len.saturating_sub(1)))` とし、空なら最後に `selected = None` にします。`Clear` は項目を消さず選択だけ消します。
