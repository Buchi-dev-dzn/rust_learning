最初の山場は「文字位置 (col) → バイト位置」の変換。`String::insert` / `remove` はバイト位置を取る。

```rust
fn byte_idx(s: &str, col: usize) -> usize {
    s.char_indices().nth(col).map(|(i, _)| i).unwrap_or(s.len())
}
```

これがあれば `insert_char` は `line.insert(byte_idx(line, col), c)`、
`insert_newline` は `let rest = line.split_off(byte_idx(line, col)); lines.insert(row + 1, rest);` で書ける。
行の文字数は `s.chars().count()`。

まず Undo を無視して、編集・移動・検索のテストを通そう (`cargo test -p f5_editor_core word` のように絞り込める)。

---

目標列は `Option<usize>` で持つ:

```rust
fn vertical(&mut self, row: usize) {
    let goal = *self.goal_col.get_or_insert(self.col); // 初回だけ今の col を覚える
    self.row = row;
    self.col = goal.min(self.line_len(row));
}
```

縦移動以外の操作では `self.goal_col = None`。

単語移動は 3 種類の分類関数 `fn class(c: char) -> Class` を作り、
行を `Vec<char>` にしてインデックスで前後に歩くと素直に書ける。

---

Undo は**スナップショット方式**が一番簡単: 状態 `State { lines, row, col }` を丸ごと保存する。

- 新しいグループを始める編集の**直前**に、今の状態を `undo_stack` に push する。合流するときは push しない。
- 編集のたびに `redo_stack.clear()`。
- `undo`: `undo_stack` から pop した状態に戻す。その前に今の状態を `redo_stack` に push。`redo` はその逆。

合流の判定には「直前の操作の種類」を持っておく:

```rust
enum LastEdit { Insert { after_space: bool }, Backspace, Delete }
last: Option<LastEdit>,   // 移動・改行・insert_str・undo/redo の後は None
```

`insert_char` は `matches!(self.last, Some(LastEdit::Insert { after_space: false }))` なら合流。
挿入後に `self.last = Some(LastEdit::Insert { after_space: c.is_whitespace() })`。
何も変えない操作 (`(0,0)` の backspace 等) は、早めに `return false` して `last` を触らないこと。
