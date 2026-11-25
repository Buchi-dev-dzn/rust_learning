# minigrep: 自作 grep

ファイルの中から文字列を含む行を探して表示する、小さな `grep` を作ります。
小さいながらも「引数を受け取り、ファイルを読み、結果を標準出力に、エラーを標準エラーに出し、
終了コードで成否を伝える」という **Unix 流 CLI ツールの基本形** がすべて入っています。

## 完成イメージ

```text
$ cargo run -p p01_minigrep -- -n fn projects/p01_minigrep/src/lib.rs
30:    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
...  (行番号:本文 の形式で一致行が並ぶ)
$ IGNORE_CASE=1 cargo run -p p01_minigrep -- rust poem.txt
$ cargo run -p p01_minigrep -- foo /no/such/file
minigrep: /no/such/file: No such file or directory (os error 2)
$ echo $?
2
```

| オプション / 環境変数 | 意味 |
|---|---|
| `-i`, `--ignore-case` | 大文字小文字を区別しない |
| `-n`, `--line-number` | 行番号を付けて表示 |
| `IGNORE_CASE` (環境変数) | 設定されていれば `-i` と同じ |

終了コード: `0` = 一致あり、`1` = 一致なし、`2` = エラー (本物の grep と同じ)。

## 学ぶこと

- `std::env::args` / `std::env::var_os` で引数と環境変数を読む
- `Result` と `?`、独自エラー型 (`enum` + `Display` + `Error`)、`Box<dyn Error>`
- ライフタイム付きの戻り値 (`Vec<Match<'a>>`) — 元の文字列を借用したまま返す
- ロジックを `lib.rs`、入出力を `main.rs` に分けてテスト可能にする設計
- `println!` (stdout) と `eprintln!` (stderr) の使い分け、`ExitCode`

作業は `projects/p01_minigrep/starter/` で行います。
テストは `cargo test -p p01_minigrep_starter`、実行は `cargo run -p p01_minigrep_starter -- <query> <file>` です。
最初はほとんどのテストが `todo!()` でパニックしますが、STEP を進めるごとに通っていきます。

## STEP 1: プロジェクトの構造を読む

やること
- `starter/src/lib.rs` と `starter/src/main.rs` を読み、どの関数が何を担当するかを把握する。
- `main.rs` は **薄く** 保つのが方針です。`main` がやるのは「引数を集める → `Config::build` → `run` → 結果を表示・終了コード」だけ。
  検索のような純粋なロジックは `lib.rs` に置き、`#[cfg(test)]` でテストします。

ポイント
- バイナリ crate (`main.rs`) とライブラリ crate (`lib.rs`) は同じパッケージに同居できます。
  `main.rs` からは `use p01_minigrep_starter::{Config, run};` のようにパッケージ名で参照します。
- 端末やファイルに依存する処理を外側に、計算を内側に置くこの分け方は、後の TUI プロジェクトでもそのまま使います。

確認方法
- `cargo build -p p01_minigrep_starter` が通ること (警告は出てよい)。
- `cargo test -p p01_minigrep_starter` を実行し、`step2_...` などのテストが `not yet implemented` で失敗するのを見ておく。

## STEP 2: 引数を Config に変換する

やること
- `Config::build` で、引数から `query` と `file_path` を取り出す。足りなければ `ArgsError` を返す。
- `ArgsError` の `Display` を実装し、人間向けのメッセージを出す。
- `main.rs` で `unwrap()` をやめ、エラー時は stderr にメッセージと `USAGE` を出して終了コード 2 を返す。

コード例
```rust,noplayground
let mut positional = Vec::new();
for arg in args {
    positional.push(arg);
}
let mut it = positional.into_iter();
let query = it.next().ok_or(ArgsError::MissingQuery)?;
let file_path = it.next().ok_or(ArgsError::MissingFile)?;
if it.next().is_some() {
    return Err(ArgsError::TooManyArgs);
}
```

```rust,noplayground
impl fmt::Display for ArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ArgsError::MissingQuery => write!(f, "検索文字列が指定されていません"),
            ArgsError::MissingFile => write!(f, "ファイル名が指定されていません"),
            ArgsError::UnknownOption(o) => write!(f, "不明なオプションです: {o}"),
            ArgsError::TooManyArgs => write!(f, "引数が多すぎます"),
        }
    }
}
```

```rust,noplayground
let config = match Config::build(env::args().skip(1), ignore_case_env) {
    Ok(c) => c,
    Err(e) => {
        eprintln!("minigrep: {e}");
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    }
};
```

ポイント
- `env::args()` の先頭はプログラム名なので `.skip(1)` します。
- `Option::ok_or` で `Option` を `Result` に変換すると `?` が使えます。
- `build` が `Vec<String>` ではなく `IntoIterator<Item = String>` を受け取るので、テストでは普通の `Vec` を渡せます。

確認方法
- `cargo test -p p01_minigrep_starter step2` が通る。
- `cargo run -p p01_minigrep_starter` (引数なし) で日本語のエラーと使い方が表示される。

## STEP 3: 検索してファイルを読む

やること
- `search` を実装する: 各行を調べ、`query` を含む行を `Match { line_no, line }` として集める。
- `run` を実装する: `fs::read_to_string` でファイルを読み、`search` し、`format_match` で文字列にする。

コード例
```rust,noplayground
pub fn search<'a>(query: &str, contents: &'a str) -> Vec<Match<'a>> {
    contents
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(query))
        .map(|(i, line)| Match { line_no: i + 1, line })
        .collect()
}
```

```rust,noplayground
pub fn run(config: &Config) -> Result<Vec<String>, Box<dyn Error>> {
    let contents = fs::read_to_string(&config.file_path)
        .map_err(|e| format!("{}: {e}", config.file_path))?;
    let matches = search(&config.query, &contents);
    Ok(matches.iter().map(|m| format_match(m, config.line_numbers)).collect())
}
```

ポイント
- 戻り値の `Match<'a>` は `contents` の一部を **借用** しています。ライフタイム `'a` は
  「返す参照は `contents` と同じだけ生きる (query とは無関係)」とコンパイラに伝えるものです。
- `run` の中で `contents` は関数ローカルなので、`Match` をそのまま返すことはできません
  (借用元が先に消えるため)。だから `String` に変換してから返しています。試しに `Vec<Match>` を返してエラーを読んでみましょう。
- `format!(...)` で作った `String` は `?` によって自動的に `Box<dyn Error>` に変換されます。

確認方法
- `cargo test -p p01_minigrep_starter step3` が通る。
- `cargo run -p p01_minigrep_starter -- fn projects/p01_minigrep/starter/src/lib.rs` で `fn` を含む行が出る。

## STEP 4: 大文字小文字を無視する (-i と IGNORE_CASE)

やること
- `search_case_insensitive` を実装する。
- `Config::build` で `-i` / `--ignore-case` を認識し、`ignore_case_env` も反映する。`-` で始まる不明なオプションは `UnknownOption` に。
- `main.rs` で `env::var_os("IGNORE_CASE").is_some()` を渡す。
- `run` で `config.ignore_case` に応じて検索関数を切り替える。

コード例
```rust,noplayground
let mut ignore_case = ignore_case_env;
let mut line_numbers = false;
let mut positional = Vec::new();
for arg in args {
    match arg.as_str() {
        "-i" | "--ignore-case" => ignore_case = true,
        "-n" | "--line-number" => line_numbers = true,
        s if s.starts_with('-') && s.len() > 1 => return Err(ArgsError::UnknownOption(arg)),
        _ => positional.push(arg),
    }
}
```

ポイント
- `match arg.as_str()` で `String` を `&str` のパターンと比較できます。ガード `s if ...` も便利。
- 環境変数を `build` の中で直接読まず **引数で渡す** ことで、テストが環境に左右されなくなります。
- `to_lowercase()` は新しい `String` を作ります (`contains` に渡すときは `&query`)。

確認方法
- `cargo test -p p01_minigrep_starter step4` が通る。
- `IGNORE_CASE=1 cargo run -p p01_minigrep_starter -- RUST <ファイル>` と `-i` 付きで同じ結果になる。

## STEP 5: 行番号を表示する (-n)

やること
- `format_match` で `line_numbers` が true のとき `"行番号:本文"` を返す。

コード例
```rust,noplayground
pub fn format_match(m: &Match<'_>, line_numbers: bool) -> String {
    if line_numbers {
        format!("{}:{}", m.line_no, m.line)
    } else {
        m.line.to_string()
    }
}
```

確認方法
- `cargo test -p p01_minigrep_starter` の全テストが通る。
- `cargo run -p p01_minigrep_starter -- -n fn projects/p01_minigrep/starter/src/lib.rs`

## STEP 6: 終了コードと stderr を仕上げる

やること
- `main` の `run(&config).unwrap()` を `match` に置き換える。
  - 一致なし → 何も出さず `ExitCode::from(1)`
  - 一致あり → 各行を `println!`、`ExitCode::SUCCESS`
  - エラー → `eprintln!("minigrep: {e}")`、`ExitCode::from(2)`

コード例
```rust,noplayground
match run(&config) {
    Ok(lines) if lines.is_empty() => ExitCode::from(1),
    Ok(lines) => {
        for line in lines {
            println!("{line}");
        }
        ExitCode::SUCCESS
    }
    Err(e) => {
        eprintln!("minigrep: {e}");
        ExitCode::from(2)
    }
}
```

ポイント
- 結果は stdout、エラーは stderr。こうしておくと `minigrep foo a.txt > out.txt` でもエラーが画面に見え、
  `out.txt` が汚れません。
- シェルでは `&&` / `||` や `if` が終了コードを見ています。`minigrep foo a.txt && echo found` を試してみましょう。

確認方法
- `cargo run -q -p p01_minigrep_starter -- zzz projects/p01_minigrep/starter/src/lib.rs; echo $?` → `1`
- `cargo run -q -p p01_minigrep_starter -- x /no/such/file; echo $?` → エラー表示と `2`
- `cargo run -q -p p01_minigrep_starter -- fn projects/p01_minigrep/starter/src/lib.rs > /dev/null; echo $?` → `0`
- 完成版 `cargo run -p p01_minigrep -- ...` と同じ動きになれば完成です。

## 発展課題

- ファイルを省略したら標準入力から読む (`std::io::stdin().read_to_string`)。`cat a.txt | minigrep foo` が動くように。
- 複数ファイルに対応し、`ファイル名:行番号:本文` 形式で出す。
- `-v` (一致しない行を表示)、`-c` (件数だけ表示) を追加する。
- 一致部分を ANSI エスケープ `\x1b[31m` ... `\x1b[0m` で赤くする (stdout が端末のときだけ。`std::io::IsTerminal` を調べてみよう)。
