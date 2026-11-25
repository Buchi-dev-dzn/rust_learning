`String::insert` / `String::remove` / `String::split_off` はどれも **バイト位置** を取ります。
文字位置 `col` は、用意されている `byte_index(line, col)` でバイト位置に直してから渡しましょう。

---

- `from_text`: `text.lines().map(|l| l.to_string()).collect()` で `Vec<String>`。空なら `String::new()` を push。
- `to_text`: `self.lines.join("\n")`。
- `newline`: `let rest = self.lines[row].split_off(at);` → `self.lines.insert(row + 1, rest);`
- `backspace` の行連結: `self.lines[row]` を借りたまま `self.lines[row - 1]` を可変で借りることはできません。先に `self.lines.remove(row)` で **所有権ごと取り出して** から前の行に `push_str` します。

---

```rust
fn backspace(&mut self, row: usize, col: usize) -> (usize, usize) {
    if col > 0 {
        let line = &mut self.lines[row];
        let at = byte_index(line, col - 1);
        line.remove(at);
        (row, col - 1)
    } else if row > 0 {
        let current = self.lines.remove(row);
        let prev = &mut self.lines[row - 1];
        let prev_len = prev.chars().count();
        prev.push_str(&current);
        (row - 1, prev_len)
    } else {
        (0, 0)
    }
}
```
