`text.split_whitespace()` は連続する空白を自動でまとめて単語だけを返します。
「組み立て中の行」`let mut current = String::new();` を用意し、単語ごとに
「今の行に空白 + 単語を足すと width を超えるか?」を判定しましょう。

---

- 超えるなら `lines.push(current)` して新しい行を始めます。`current` を push した後も使い続けたいので、`lines.push(std::mem::take(&mut current))` が便利です (第2章)。
- 長すぎる単語は `while word.len() > width { let (head, tail) = word.split_at(width); ... word = tail; }` で切り出します。切り出す前に組み立て中の行があれば先に push。
- `frame` は `format!("|{:<width$}|", line)` で各行を作り、上下に `format!("+{}+", "-".repeat(width))`。

---

```rust
for word in text.split_whitespace() {
    let mut word = word;
    while word.len() > width {
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        let (head, tail) = word.split_at(width);
        lines.push(head.to_string());
        word = tail;
    }
    if word.is_empty() {
        continue;
    }
    let needed = if current.is_empty() { word.len() } else { current.len() + 1 + word.len() };
    if needed > width {
        lines.push(std::mem::take(&mut current));
    }
    if !current.is_empty() {
        current.push(' ');
    }
    current.push_str(word);
}
if !current.is_empty() {
    lines.push(current);
}
```
