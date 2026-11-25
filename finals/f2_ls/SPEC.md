# F2: カラー対応の ls

ディレクトリの中身を、**端末の幅に合わせた複数列**で表示する `ls` を作る。
ディレクトリ・実行ファイル・シンボリックリンクは ANSI カラーで色分けし、`-l` で詳細表示もできる。

```
f2_ls [-a] [-l] [PATH...]
```

この課題で身につくこと: `std::fs` によるディレクトリ走査とメタデータ、
**Unicode の表示幅** (全角文字は 2 セル)、ANSI エスケープによる色付け、
「整形ロジック」と「I/O」の分離。TUI で文字列を正しく並べるための基礎体力になる。

外部クレートは `libc` のみ使用可 (端末幅の取得・isatty 用)。Unicode 表示幅も自作する。

## 1. 型 (`src/lib.rs`)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind { File, Dir, Symlink, Executable }

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub kind: Kind,
    pub mode: u32,            // パーミッションビット (下位 9 ビット, 0o755 など)
    pub size: u64,            // バイト数 (シンボリックリンクはリンク自体のサイズ)
    pub nlink: u64,           // ハードリンク数
    pub link_target: Option<String>, // シンボリックリンクのリンク先 (それ以外は None)
}

impl Entry {
    pub fn new(name: &str, kind: Kind) -> Entry;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Options { pub all: bool, pub long: bool, pub color: bool }
```

`Entry::new` の既定値: `mode` は Dir/Executable → `0o755`, Symlink → `0o777`, File → `0o644`。
`size = 0`, `nlink = 1`, `link_target = None`。

## 2. 関数

```rust
pub fn char_width(c: char) -> usize;
pub fn str_width(s: &str) -> usize;
pub fn pad_right(s: &str, width: usize) -> String;
pub fn colorize(entry: &Entry) -> String;
pub fn format_grid(entries: &[Entry], term_width: usize, color: bool) -> Vec<String>;
pub fn mode_string(entry: &Entry) -> String;
pub fn format_long(entries: &[Entry], color: bool) -> Vec<String>;
pub fn read_entries(dir: &Path, all: bool) -> std::io::Result<Vec<Entry>>;
pub fn parse_args(args: &[String]) -> Result<(Options, Vec<String>), String>;
```

### `char_width(c)` — 表示幅

| 条件 (コードポイント) | 幅 |
|---|---|
| `< 0x20`、`0x7F..=0x9F` (制御文字) | 0 |
| `0x0300..=0x036F` (結合文字)、`0x200B` | 0 |
| 下の「全角範囲」のどれか | 2 |
| それ以外 | 1 |

全角範囲: `1100-115F`, `2E80-303E`, `3041-33FF`, `3400-4DBF`, `4E00-9FFF`, `A000-A4CF`,
`AC00-D7A3`, `F900-FAFF`, `FE30-FE4F`, `FF00-FF60`, `FFE0-FFE6`, `1F300-1F64F`,
`1F900-1F9FF`, `20000-3FFFD` (すべて両端を含む)。

`str_width(s)` は各文字の `char_width` の合計。
`pad_right(s, w)` は表示幅が `w` になるまで右に空白を足す (すでに `w` 以上ならそのまま)。

### `colorize(entry)`

| Kind | 出力 |
|---|---|
| Dir | `"\x1b[1;34m" + name + "\x1b[0m"` (太字青) |
| Executable | `"\x1b[1;32m" + name + "\x1b[0m"` (太字緑) |
| Symlink | `"\x1b[1;36m" + name + "\x1b[0m"` (太字シアン) |
| File | `name` そのまま |

### `format_grid(entries, term_width, color)` — 複数列表示

GNU ls と同じ **列優先** (上から下へ、次に右の列へ) で並べる。順序は `entries` の順のまま。

1. `n = entries.len()`。`n == 0` なら空の `Vec`。
2. 列数の候補 `c` を `n` から 1 まで減らしながら試す:
   - `rows = ceil(n / c)`、実際の列数 `cols = ceil(n / rows)`。
   - 列 `j` には `entries[j*rows .. min((j+1)*rows, n)]` が入る。列幅 = その列の名前の表示幅の最大値。
   - 合計幅 = 列幅の合計 + `2 * (cols - 1)` (列間は空白 2 つ)。
   - 合計幅 `<= term_width` なら、この配置を採用して終了。`c == 1` なら幅を超えても採用。
3. 各行は、その行に属する名前を左から並べたもの。**行の最後の名前以外**は
   「名前 + 空白」で `列幅 + 2` の表示幅になるまで埋める。**行末に空白は付けない**。
4. `color == true` のとき、名前は `colorize` の結果を使う。**幅の計算は色なしの名前で行う**
   (エスケープシーケンスは画面上の幅を持たない)。

例: 名前 `a bb ccc dddd eeeee` (5 個), `term_width = 20`:
`c=5` → 幅 1+2+3+4+5+8=23 > 20、`c=4` → rows=2, cols=3, 列幅 [2,4,5] → 2+4+5+4 = 15 ≤ 20 → 採用。

```
a   ccc   eeeee
bb  dddd
```

### `mode_string(entry)`

10 文字。1 文字目: Dir → `d`, Symlink → `l`, それ以外 → `-`。
続く 9 文字は `mode` の 0o400, 0o200, 0o100, 0o040, … 0o001 のビットを `rwxrwxrwx` で表し、立っていなければ `-`。
例: Dir, 0o755 → `drwxr-xr-x`。

### `format_long(entries, color)` — `-l`

各エントリ 1 行:

```
{mode_string} {nlink:>L} {size:>S} {名前}
```

- `L` = 全エントリの `nlink` の 10 進表記の最大桁数、`S` = `size` の最大桁数 (右寄せ)。
- 名前は `color` なら `colorize`、そうでなければそのまま。
- `link_target` が `Some(t)` なら末尾に `" -> " + t`。

例: `drwxr-xr-x 2    64 sub` / `-rw-r--r-- 1 12345 a.txt`

### `read_entries(dir, all)`

- `dir` 直下のエントリを読む (`.` と `..` は含めない)。
- `all == false` のとき、`.` で始まる名前を除外する。
- **シンボリックリンクは辿らない** (`symlink_metadata` を使う)。
- Kind の判定順: シンボリックリンク → `Symlink` (`link_target` = `read_link` の結果)、
  ディレクトリ → `Dir`、`mode & 0o111 != 0` → `Executable`、それ以外 → `File`。
- `mode` はパーミッションの下位 9 ビット (`& 0o777`)。
- 結果は `name` の**バイト順 (String の `cmp`)** でソート。
- ディレクトリが読めなければ `Err`。

### `parse_args(args)` (`args` はプログラム名を除く)

- `-` で始まる 2 文字以上の引数はオプション。`-a` `-l` `-al` `-la` などを受け付ける。
  その他の文字 → `Err(format!("invalid option: -{ch}"))`。
- それ以外はパス。パスが 1 つもなければ `["."]`。
- `Options.color` は常に `false` で返す (色を使うかどうかは main が `isatty` で決める)。

## 3. `src/main.rs`

- 端末幅: `libc::ioctl(STDOUT_FILENO, TIOCGWINSZ, ..)` → 環境変数 `COLUMNS` → 80 の順。
- 標準出力が端末 (`libc::isatty(1) == 1`) のときだけ色を付ける。
- パスが複数ならそれぞれの前に `PATH:` を出し、間に空行を入れる。
- 読めないパスは `f2_ls: PATH: エラー` を stderr に出して終了コード 1。不正なオプションは終了コード 2。

main.rs はテストされないが、`cargo run -p f2_ls -- -l ~` などで動作確認しよう。

## 合格条件

`cargo test -p f2_ls` がすべて成功すること。
