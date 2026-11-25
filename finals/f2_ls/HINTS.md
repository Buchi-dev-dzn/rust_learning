小さい関数から順に: `Entry::new` → `char_width` / `str_width` / `pad_right` → `colorize` → `mode_string`。
ここまでは数行ずつで書ける。

`char_width` は `matches!` マクロで範囲をまとめると読みやすい:

```rust
let u = c as u32;
if u < 0x20 || (0x7F..=0x9F).contains(&u) { return 0; }
if matches!(u, 0x1100..=0x115F | 0x2E80..=0x303E /* ... */) { 2 } else { 1 }
```

`mode_string` は `[(0o400, 'r'), (0o200, 'w'), (0o100, 'x'), ...]` の配列を回すと 1 ループで書ける。

---

`format_grid` は「配置を決める」と「文字列を組み立てる」の 2 段に分けよう。

配置: `for c in (1..=n).rev()` で列数を試し、`rows = n.div_ceil(c)`、`cols = n.div_ceil(rows)`。
列 `j` の幅は `widths[j*rows .. ((j+1)*rows).min(n)]` の最大値。

組み立て: 行 `r`、列 `j` の要素は `entries[j * rows + r]` (範囲外なら、その行はそこで終わり)。
「この要素が行の最後か」は `j + 1 == cols || (j + 1) * rows + r >= n` で判定できる。

色付きのときも、パディング量は**色なしの名前の幅** (`str_width(&e.name)`) から計算すること。
`colorize` の結果に `pad_right` をかけると、ESC (0x1B) 自体は幅 0 でも、続く `[1;34m` などの
文字が幅 1 として数えられて列がずれる。必ず名前の幅を使う。

---

`read_entries`:

```rust
use std::os::unix::fs::{MetadataExt, PermissionsExt};

for item in std::fs::read_dir(dir)? {
    let item = item?;
    let name = item.file_name().to_string_lossy().into_owned();
    let meta = std::fs::symlink_metadata(item.path())?; // リンクを辿らない
    let ft = meta.file_type();
    let mode = meta.permissions().mode() & 0o777;
    // ft.is_symlink() → read_link, ft.is_dir(), mode & 0o111 != 0 の順で Kind を決める
    // size = meta.len(), nlink = meta.nlink()
}
entries.sort_by(|a, b| a.name.cmp(&b.name));
```

`format_long` は先に `nlink` / `size` を文字列化して最大桁数を求め、`format!("{:>w$}", x, w = width)` で右寄せする。
