# F1: wc / cat / head を自作

`wc` (行・単語・文字・バイト数), `head` (先頭 N 行), `cat` (連結・行番号付き) を
**1 つのバイナリのサブコマンド**として実装する。標準ライブラリのみを使う。

```
f1_coreutils wc   [-l] [-w] [-m] [-c] [FILE...]
f1_coreutils head [-n N] [FILE...]
f1_coreutils cat  [-n] [FILE...]
```

この課題で身につくこと: `std::io::{Read, Write}` を引数に取る「テストしやすい I/O 設計」、
引数解析、エラー処理と終了コード、バイト列 (`&[u8]`) と UTF-8 の違い。

## ファイル構成

- `src/lib.rs` … ロジック (下記の関数をすべて `pub` で公開する)
- `src/main.rs` … `run` を呼んで終了コードを返すだけ (starter に同梱済み)
- `tests/acceptance.rs` … 受け入れテスト (編集しない)

## 1. 公開 API (`src/lib.rs`)

テストは以下のシグネチャに依存する。名前・型を変えないこと。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Counts {
    pub lines: usize,
    pub words: usize,
    pub chars: usize,
    pub bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WcFlags {
    pub lines: bool,
    pub words: bool,
    pub chars: bool,
    pub bytes: bool,
}

pub fn count(input: &[u8]) -> Counts;
pub fn format_counts(counts: &Counts, flags: &WcFlags, name: Option<&str>) -> String;
pub fn head_lines(input: &[u8], n: usize) -> &[u8];
pub fn number_lines(input: &[u8], start: usize) -> (Vec<u8>, usize);
pub fn run(args: &[String], stdin: &mut dyn Read, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32;
```

### `count(input)`

- `lines` … `b'\n'` の個数 (最後の行に改行がなければその行は数えない。GNU wc と同じ)。
- `words` … 空白で区切られた「空白でないバイトの連続」の個数。
  空白 = `b' '`, `b'\t'`, `b'\n'`, `b'\r'`, `0x0B`, `0x0C` の 6 種。
- `bytes` … `input.len()`。
- `chars` … UTF-8 の文字数。**継続バイト (`b & 0xC0 == 0x80`) 以外のバイト数** と定義する
  (不正な UTF-8 でもパニックしない)。

### `format_counts(counts, flags, name)`

- 出力するフィールドの順序は常に **lines, words, chars, bytes**。
- `flags` がすべて `false` のときは `lines, words, bytes` を出力する (デフォルト)。
- 各数値は `format!("{:>8}", n)` で右寄せ幅 8。それを連結する (区切り文字なし)。
- `name` が `Some(s)` なら末尾に `" "` + `s` を付ける。末尾に改行は**付けない**。

例: lines=3, words=5, bytes=20, name=`a.txt` → `"       3       5      20 a.txt"`

### `head_lines(input, n)`

先頭 `n` 行 (改行を含む) からなる `input` の先頭部分スライスを返す。
行数が `n` 未満なら `input` 全体。`n == 0` なら空スライス。

### `number_lines(input, start)`

`cat -n` 相当。各行の先頭に `format!("{:>6}\t", 行番号)` を付ける。
行番号は `start` から始まる。戻り値は `(出力, 次に使う行番号)`。

- 最後の行に改行がない場合も番号を付け、改行は**追加しない**。
- 空入力 → `(空, start)`。
- 例: `number_lines(b"a\nb", 1)` → `(b"     1\ta\n     2\tb".to_vec(), 3)`

## 2. `run` (= コマンド全体)

`args` は**プログラム名を除いた**引数 (`["wc", "-l", "a.txt"]` など)。
`stdin` / `stdout` / `stderr` に読み書きし、終了コードを返す。
`main.rs` は `std::process::exit(run(...))` するだけ。

### 共通規則

- `args` が空、またはサブコマンドが `wc` / `head` / `cat` 以外 →
  stderr に `usage: f1_coreutils <wc|head|cat> [OPTIONS] [FILE...]` + 改行を書き、**終了コード 2**。
- `-` で始まり 2 文字以上の引数はオプション (位置はどこでもよい)。それ以外はファイル名。
- 1 文字の `-` は「標準入力」を意味するファイル名。
- ファイル引数が 1 つもなければ標準入力を 1 回読む。
- 未知のオプション → stderr に `f1_coreutils: invalid option: <arg>` + 改行、**終了コード 2** (何も出力しない)。
- ファイルが開けない/読めない → stderr に `f1_coreutils: <path>: <エラー内容>` + 改行を書き、
  **残りのファイルの処理は続ける**。最終的な終了コードは 1。
- すべて成功すれば終了コード 0。

### `wc`

- オプション `-l` `-w` `-m` `-c` (それぞれ lines/words/chars/bytes)。`-lw` のようにまとめてもよい。
- 各入力について `format_counts` の結果 + `"\n"` を出力。
  ファイル名はファイルのとき `Some(path)`、標準入力のとき `None`。
- **ファイル引数が 2 つ以上**のとき、最後に成功したファイルの合計を `Some("total")` で出力する。

### `head`

- `-n N` または `-nN` で行数指定 (デフォルト 10)。
  `N` が非負整数でない/値がない → stderr に `f1_coreutils: invalid number of lines: <値>` + 改行、終了コード 2
  (値がないときの `<値>` は空文字列)。
- **ファイル引数が 2 つ以上**のとき、各ファイルの前にヘッダ `==> <path> <==\n` を出す。
  2 つ目以降のヘッダの前には空行 `\n` を入れる (開けなかったファイルはヘッダも出さない)。
  標準入力 (`-`) の名前は `standard input`。

### `cat`

- オプションなし: 入力をそのまま連結して出力。
- `-n`: `number_lines` で番号付け。**番号はファイルをまたいで続く**。

## 3. 例

```
$ printf 'hello world\nfoo\n' | f1_coreutils wc
       2       3      16
$ f1_coreutils head -n 1 a.txt b.txt
==> a.txt <==
line1

==> b.txt <==
first
$ f1_coreutils cat -n a.txt
     1	line1
     2	line2
$ f1_coreutils frobnicate ; echo $?
usage: f1_coreutils <wc|head|cat> [OPTIONS] [FILE...]
2
```

## 合格条件

`cargo test -p f1_coreutils` がすべて成功すること。
