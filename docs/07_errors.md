# 第7章 エラー処理 (Result, ?, 独自エラー型)

## この章のゴール

- 回復不能なエラー (`panic!`) と回復可能なエラー (`Result`) を使い分けられる
- `Result<T, E>` を `match` / `?` / `map_err` / `unwrap_or` などで扱える
- 独自のエラー enum を定義し、`Display` と `std::error::Error` を実装できる
- `From` を実装して `?` でエラー型を自動変換できる
- `main` から `Result` を返し、`Box<dyn Error>` で手早くエラーをまとめられる

ターミナルアプリは失敗だらけの世界で動きます。ファイルがない、引数がおかしい、
端末がサイズを教えてくれない、書き込みに失敗する…。
Rust はこれらを **戻り値の型** で表現し、扱い忘れをコンパイル時に防ぎます。

## panic!: 回復不能なエラー

```rust
fn main() {
    let v = vec![1, 2, 3];
    let i = 1;
    if i >= v.len() {
        panic!("添字 {i} は範囲外です"); // プログラムを停止する
    }
    println!("{}", v[i]);
}
```

`panic!` はスレッドを巻き戻して (unwind) 停止します。使うのは次のような場合です。

- プログラムのバグ (起こるはずのない状態)。
- テストの失敗 (`assert!` も内部で panic します)。
- プロトタイプで、エラー処理を後回しにしたいとき (`unwrap` / `expect`)。

TUI では、パニックが起きると **ターミナルが raw モードのまま壊れた状態で残る** という問題があります。
第19章でパニック時にも端末を元に戻す仕組み (パニックフック、Drop) を作ります。
普段のエラーは panic ではなく `Result` で返すのが基本です。

## Result<T, E>

```rust
// 標準ライブラリの定義
enum Result<T, E> {
    Ok(T),   // 成功。値 T を持つ
    Err(E),  // 失敗。エラー E を持つ
}
```

```rust
fn main() {
    let inputs = ["42", "-7", "abc", ""];
    for s in inputs {
        // str::parse は Result<T, ParseIntError> を返す
        match s.parse::<i32>() {
            Ok(n) => println!("{s:?} → {n}"),
            Err(e) => println!("{s:?} → エラー: {e}"),
        }
    }
}
```

`Result` は `#[must_use]` なので、戻り値を無視すると警告が出ます。

### Result を扱うメソッド

```rust
fn main() {
    let good: Result<u16, String> = Ok(80);
    let bad: Result<u16, String> = Err("不正な値".to_string());

    println!("{} {}", good.clone().unwrap_or(0), bad.clone().unwrap_or(0));
    println!("{:?}", good.clone().map(|w| w * 2));                 // Ok(160)
    println!("{:?}", bad.clone().map_err(|e| format!("設定: {e}"))); // エラー側を変換
    println!("{:?} {:?}", good.clone().ok(), bad.clone().ok());    // Option に変換
    println!("{} {}", good.is_ok(), bad.is_err());
    let w = good.clone().unwrap_or_else(|e| {
        eprintln!("警告: {e}");
        80
    });
    println!("{w}");
    // Option → Result
    let maybe: Option<u16> = None;
    println!("{:?}", maybe.ok_or("値がありません"));
}
```

| メソッド | 動作 |
|----------|------|
| `unwrap()` / `expect("msg")` | `Ok` なら中身、`Err` なら panic |
| `unwrap_or(v)` / `unwrap_or_else(f)` / `unwrap_or_default()` | `Err` なら代わりの値 |
| `map(f)` / `map_err(f)` | `Ok` 側 / `Err` 側を変換 |
| `and_then(f)` | `Ok` なら次の `Result` を返す処理へつなぐ |
| `ok()` | `Result<T, E>` → `Option<T>` |
| `opt.ok_or(e)` | `Option<T>` → `Result<T, E>` |

## ? 演算子: エラーの伝播

「エラーなら呼び出し元にそのまま返し、成功なら中身を取り出す」を 1 文字で書けるのが `?` です。

```rust
use std::num::ParseIntError;

fn parse_size(s: &str) -> Result<(u16, u16), ParseIntError> {
    let (w, h) = s.split_once('x').unwrap_or((s, ""));
    let w: u16 = w.trim().parse()?; // Err なら即 return Err(..)
    let h: u16 = h.trim().parse()?;
    Ok((w, h))
}

fn main() {
    println!("{:?}", parse_size("80x24"));
    println!("{:?}", parse_size("80xabc"));
    println!("{:?}", parse_size("80"));
}
```

`let w = s.parse()?;` は次の `match` とほぼ同じ意味です。

```rust
fn demo(s: &str) -> Result<u16, std::num::ParseIntError> {
    let w: u16 = match s.parse() {
        Ok(v) => v,
        Err(e) => return Err(e.into()), // .into() で戻り値のエラー型へ変換 (後述)
    };
    Ok(w)
}
```

- `?` は **戻り値が `Result` (または `Option`) の関数の中でしか使えません**。
- `Option` に `?` を使うと、`None` のとき `None` を返します。

## 独自のエラー型

複数の種類のエラーが起きる関数では、エラーを enum で定義するのが Rust の流儀です。

```rust
use std::fmt;

#[derive(Debug, PartialEq)]
enum ConfigError {
    MissingEquals { line: usize },
    UnknownKey(String),
    BadValue { key: String, value: String },
}

// ユーザー向けの表示 ({} で使われる)
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ConfigError::MissingEquals { line } => write!(f, "{line} 行目: '=' がありません"),
            ConfigError::UnknownKey(k) => write!(f, "不明な設定項目: {k}"),
            ConfigError::BadValue { key, value } => write!(f, "{key} の値が不正: {value}"),
        }
    }
}

// これで「エラー型」として扱える (Box<dyn Error> に入れられる)
impl std::error::Error for ConfigError {}

#[derive(Debug, Default)]
struct Config {
    tab_width: u8,
    line_numbers: bool,
}

fn parse_config(text: &str) -> Result<Config, ConfigError> {
    let mut cfg = Config::default();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or(ConfigError::MissingEquals { line: i + 1 })?;
        let (key, value) = (key.trim(), value.trim());
        let bad = || ConfigError::BadValue { key: key.to_string(), value: value.to_string() };
        match key {
            "tab_width" => cfg.tab_width = value.parse().map_err(|_| bad())?,
            "line_numbers" => cfg.line_numbers = value.parse().map_err(|_| bad())?,
            other => return Err(ConfigError::UnknownKey(other.to_string())),
        }
    }
    Ok(cfg)
}

fn main() {
    let samples = [
        "tab_width = 4\nline_numbers = true",
        "tab_width = 4\noops",
        "color = red",
        "tab_width = huge",
    ];
    for s in samples {
        match parse_config(s) {
            Ok(cfg) => println!("OK: {cfg:?}"),
            Err(e) => println!("エラー: {e}"),
        }
    }
}
```

ポイント:

- `Debug` は開発者向け、`Display` はユーザー向けの表示です。エラーには両方を用意します。
- `impl std::error::Error for ConfigError {}` は中身が空でかまいません (`Debug + Display` があれば十分)。
- `map_err` で下位のエラー (`ParseIntError` など) を自分のエラーに変換してから `?` します。

## From と ? による自動変換

`?` は、エラーを返すときに `From::from` で **戻り値のエラー型へ自動変換** します。
`From` を実装しておけば `map_err` を書かなくて済みます。

```rust
use std::fmt;
use std::num::ParseIntError;

#[derive(Debug)]
enum AppError {
    Parse(ParseIntError),
    OutOfRange(i64),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Parse(e) => write!(f, "数値として読めません ({e})"),
            AppError::OutOfRange(n) => write!(f, "{n} は範囲外です (1〜100)"),
        }
    }
}

impl std::error::Error for AppError {}

impl From<ParseIntError> for AppError {
    fn from(e: ParseIntError) -> Self {
        AppError::Parse(e)
    }
}

fn parse_percent(s: &str) -> Result<u8, AppError> {
    let n: i64 = s.trim().parse()?; // ParseIntError → AppError へ自動変換
    if !(1..=100).contains(&n) {
        return Err(AppError::OutOfRange(n));
    }
    Ok(n as u8)
}

fn main() {
    for s in ["50", "abc", "200"] {
        match parse_percent(s) {
            Ok(p) => println!("{p}%"),
            Err(e) => println!("エラー: {e}"),
        }
    }
}
```

## main から Result を返す

`main` は `Result<(), E>` を返せます。`Err` を返すと、エラーの `Debug` 表示が出力され、
終了コード 1 で終了します。

```rust
use std::error::Error;

fn run() -> Result<(), Box<dyn Error>> {
    let n: i32 = "12".parse()?;              // ParseIntError
    let size: u16 = u16::try_from(n * 1000)?; // TryFromIntError
    println!("size = {size}");
    let bad: i32 = "x".parse()?;             // ここで失敗して return
    println!("ここには来ない {bad}");
    Ok(())
}

fn main() {
    // 実際のアプリでは fn main() -> Result<(), Box<dyn Error>> とすることも多い。
    // ここでは終了コードを 0 にするため、自分でエラーを表示する。
    if let Err(e) = run() {
        eprintln!("エラー: {e}");
    }
}
```

`Box<dyn Error>` は「何らかのエラー型」を入れられる箱です。どんな `Error` 型からも `?` で変換されるので、
アプリの上位層やプロトタイプでは手軽に使えます。
一方、ライブラリやテストで「どの種類のエラーか」を判別したい場所では、独自の enum を使いましょう。
(よく使われる外部クレート `anyhow` / `thiserror` は、それぞれこの 2 つの書き方を楽にするものです)

終了コードを細かく制御したいときは `std::process::exit(2)` や `std::process::ExitCode` を使います (第15章)。

## どこで何を使うか

| 状況 | 使うもの |
|------|----------|
| 起こり得る失敗 (入力ミス、ファイルなし、I/O) | `Result` を返す |
| 値がないことが普通にある (検索結果なし) | `Option` を返す |
| バグ・前提条件違反 | `panic!` / `assert!` / `unreachable!` |
| テストコード・使い捨てスクリプト | `unwrap` / `expect` で OK |
| ライブラリの公開 API | 独自のエラー enum |
| アプリの main 付近 | `Box<dyn Error>` でまとめて表示 |

`expect` を使う場合は「なぜ失敗しないと言えるか」をメッセージに書くと、後で読む人の助けになります。

## よくあるコンパイルエラーと読み方

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option`
```

`?` を `()` を返す関数 (例えば普通の `fn main()`) で使いました。関数の戻り値を `Result` にします。

```text
error[E0277]: `?` couldn't convert the error to `AppError`
  = note: the question mark operation (`?`) implicitly performs a conversion on the error value using the `From` trait
  = help: the trait `From<ParseIntError>` is not implemented for `AppError`
```

`?` でエラーを変換しようとしたが、`From` が実装されていません。`impl From<ParseIntError> for AppError` を書くか、
`map_err` で明示的に変換します。

| コード | 状況 | 直し方 |
|--------|------|--------|
| E0277 (`?` in fn returning `()`) | `?` を使える関数ではない | 戻り値を `Result<_, _>` に |
| E0277 (`couldn't convert the error`) | `From` がない | `impl From<..>` / `map_err` |
| E0308 `expected Result, found ..` | `Ok(..)` で包み忘れ | 最後を `Ok(value)` に |
| E0277 `doesn't implement Display` | `{}` でエラーを表示 | `impl fmt::Display` |
| warning `unused Result` | 戻り値の `Result` を無視 | `?` / `match` / `let _ =` (意図的に無視する場合) |

## まとめ

- バグは `panic!`、起こり得る失敗は `Result`。
- `?` は「`Err` なら早期リターン、`Ok` なら中身」。エラーは `From` で自動変換される。
- 独自エラーは enum で定義し、`Debug` / `Display` / `Error` を実装する。
- `main` は `Result` を返せる。上位層では `Box<dyn Error>` が手軽。
- TUI では panic でターミナルが壊れるので、なおさら `Result` で丁寧に扱う。

## 次へ

第8章ではコードをモジュールに分け、Cargo でプロジェクトやクレートを管理する方法を学びます。
Stage 1 の最後の章です。
