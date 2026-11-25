テストは `crate::terminal::style::sgr` のようなパスで使っています。
このパスの途中にあるもの (モジュール `style`、関数 `sgr`、enum `Color`) が **すべて** 外から見えなければなりません。
(`terminal` 自体はクレートのルート直下にあり、`tests` から見て兄弟なので `pub` は不要です)

---

- `mod style` → `pub mod style`、`enum Color` → `pub enum Color` (バリアントは自動で公開)、`fn sgr` → `pub fn sgr`。
- `struct Screen` → `pub struct Screen`。メソッド `new` / `put` / `row_text` にも `pub`。
- テストは `s.width` と `s.height` を読んでいるので、この 2 つのフィールドに `pub`。`cells` には付けません。

---

```rust
pub struct Screen {
    pub width: usize,
    pub height: usize,
    cells: Vec<char>,
}
```

試しに、テストに `s.cells.clear();` と書き足すとどうなるか見てみましょう (E0616 になれば正解。試したら消すこと)。
