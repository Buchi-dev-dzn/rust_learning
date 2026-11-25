# 第15章 std::io・ファイル・引数・環境変数・終了コード

## この章のゴール

- 標準入力 (stdin)・標準出力 (stdout)・標準エラー出力 (stderr) の役割を説明でき、正しく使い分けられる
- `BufRead` / `Write` トレイトを使って、行単位の読み込みと効率のよい書き込みができる
- `lock()` と `BufWriter` で出力を速くする理由がわかる
- ファイルを開く・読む・書く・追記する方法を知る
- `std::env::args` でコマンドライン引数を、`std::env::var` で環境変数を読める
- `std::process::ExitCode` / `std::process::exit` で終了コードを返せる
- 「パイプにつながっているか、端末につながっているか」で挙動を変える考え方 (isatty) を知る

ここで学ぶ内容は、そのまま後半の TUI づくりの土台になります。TUI も結局は
「stdin からバイトを読み、stdout にバイトを書く」プログラムだからです。

## 3 つの標準ストリーム

Unix のプロセスは起動時に 3 つのファイルディスクリプタ (fd) を持っています。

| fd | 名前 | Rust | 用途 |
|----|------|------|------|
| 0 | 標準入力 | `std::io::stdin()` | 入力データ・キー入力 |
| 1 | 標準出力 | `std::io::stdout()` | プログラムの「結果」 |
| 2 | 標準エラー出力 | `std::io::stderr()` | エラー・ログ・進捗表示 |

大事なのは **結果は stdout、それ以外は stderr** という使い分けです。
`mytool input.txt > out.txt` のようにリダイレクトしたとき、エラーメッセージが
`out.txt` に混ざらず画面に出るのは、エラーを stderr に書いているからです。

`println!` は stdout へ、`eprintln!` は stderr へ書きます。

```rust
fn main() {
    println!("これは結果 (stdout)");
    eprintln!("これはログ (stderr)");
}
```

## Write トレイト: バイトを書く

`println!` はマクロですが、その下にあるのは `std::io::Write` トレイトです。
`Write` は「バイト列をどこかに書けるもの」を表し、stdout・ファイル・`Vec<u8>`・
TCP ソケットなどが実装しています。

主なメソッド:

- `write_all(&mut self, buf: &[u8]) -> io::Result<()>` … 全部書く
- `flush(&mut self) -> io::Result<()>` … バッファを吐き出す
- `write!(w, ...)` / `writeln!(w, ...)` マクロ … フォーマットして書く

```rust
use std::io::{self, Write};

fn main() -> io::Result<()> {
    let mut out = io::stdout();
    out.write_all(b"bytes directly\n")?;
    writeln!(out, "{} + {} = {}", 1, 2, 1 + 2)?;
    out.flush()?;
    Ok(())
}
```

`Vec<u8>` も `Write` を実装しているので、**テストでは stdout の代わりに `Vec<u8>` に書かせる**
のが定番です。関数を `impl Write` (またはジェネリクス `W: Write`) で受け取るように
書いておけば、本番では stdout、テストでは `Vec<u8>` を渡せます。

```rust
use std::io::{self, Write};

fn greet<W: Write>(w: &mut W, name: &str) -> io::Result<()> {
    writeln!(w, "Hello, {name}!")
}

fn main() -> io::Result<()> {
    // 本番: 画面へ
    greet(&mut io::stdout(), "terminal")?;

    // テスト: メモリへ
    let mut buf: Vec<u8> = Vec::new();
    greet(&mut buf, "test")?;
    assert_eq!(String::from_utf8(buf).unwrap(), "Hello, test!\n");
    println!("テスト用バッファにも正しく書けた");
    Ok(())
}
```

この「出力先を差し替えられるように書く」という設計は、TUI の描画関数をテストするときにも
そのまま使います (第22章で ANSI 出力を `String` に貯めてテストします)。

## lock と BufWriter: 出力を速くする

`io::stdout()` はプロセス全体で共有されるハンドルで、内部に `Mutex` を持っています。
`println!` は呼ぶたびにロックを取り、行ごとにフラッシュします (stdout は **行バッファリング**)。
大量に出力するときはこれが遅くなります。

対策は 2 つ:

1. `stdout().lock()` でロックを一度だけ取る
2. `BufWriter` で包んで、書き込みをまとめる

```rust
use std::io::{self, BufWriter, Write};

fn main() -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = BufWriter::new(stdout.lock());
    for i in 0..5 {
        writeln!(out, "line {i}")?;
    }
    out.flush()?; // Drop でもフラッシュされるが、エラーを拾うには明示的に
    Ok(())
}
```

TUI では 1 フレーム分の描画 (カーソル移動・色・文字) を全部バッファに溜めてから
**1 回だけ flush** します。少しずつ書くと画面がちらつくからです。
この感覚をここで掴んでおきましょう。

> 注意: 行バッファリングなので、改行を含まない `print!("> ")` はすぐには表示されません。
> プロンプトを出すときは `io::stdout().flush()` を呼びます。

## BufRead: 行単位で読む

標準入力を行ごとに読むには `BufRead` トレイトの `lines()` を使います。

```rust,noplayground
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    for line in stdin.lock().lines() {
        let line = line?; // 各行は io::Result<String>
        println!("{}", line.to_uppercase());
    }
    Ok(())
}
```

(この例は標準入力を読むので実行ボタンはありません。手元で `echo hi | cargo run` などで試してください。)

`lines()` は末尾の改行 (`\n` や `\r\n`) を取り除いた `String` を返します。
改行も含めて欲しい場合や、1 つの `String` を使い回して速くしたい場合は `read_line` を使います。

```rust,noplayground
use std::io::{self, BufRead};

fn main() -> io::Result<()> {
    let mut stdin = io::stdin().lock();
    let mut buf = String::new();
    loop {
        buf.clear();
        let n = stdin.read_line(&mut buf)?; // 改行を含む。0 なら EOF
        if n == 0 {
            break;
        }
        print!("{buf}");
    }
    Ok(())
}
```

`BufRead` も `Write` と同じく差し替え可能です。`&[u8]` (バイトスライス) や
`std::io::Cursor` は `BufRead` を実装しているので、テストでは文字列を入力として渡せます。

```rust
use std::io::{self, BufRead};

fn count_nonempty<R: BufRead>(r: R) -> io::Result<usize> {
    let mut n = 0;
    for line in r.lines() {
        if !line?.trim().is_empty() {
            n += 1;
        }
    }
    Ok(n)
}

fn main() -> io::Result<()> {
    let input = "a\n\n  \nb\nc\n";
    let n = count_nonempty(input.as_bytes())?;
    println!("空でない行: {n}");
    Ok(())
}
```

## ファイル操作

`std::fs` には便利関数がそろっています。

| やりたいこと | 関数 |
|---|---|
| 全体を文字列で読む | `fs::read_to_string(path)` |
| 全体をバイトで読む | `fs::read(path)` |
| 全体を書く (上書き) | `fs::write(path, data)` |
| 開いて少しずつ読む | `File::open(path)` + `BufReader` |
| 作成して少しずつ書く | `File::create(path)` + `BufWriter` |
| 追記 | `OpenOptions::new().append(true).create(true).open(path)` |

```rust
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Write};

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("rust_learning_15_demo.txt");

    fs::write(&path, "first\n")?;

    let mut f = OpenOptions::new().append(true).open(&path)?;
    writeln!(f, "second")?;
    drop(f); // ここでファイルが閉じられる

    let reader = BufReader::new(File::open(&path)?);
    for (i, line) in reader.lines().enumerate() {
        println!("{}: {}", i + 1, line?);
    }

    fs::remove_file(&path)?;
    Ok(())
}
```

ファイルは `File` が Drop されたときに自動で閉じられます。Rust では「スコープを抜けたら後片付け」
が基本で、第18章・第19章で **ターミナル設定を元に戻す Drop ガード** にも同じ考え方を使います。

存在しないファイルを開くと `io::Error` が返ります。`e.kind()` で種類を判別できます。

```rust
use std::fs::File;
use std::io::ErrorKind;

fn main() {
    match File::open("/no/such/file") {
        Ok(_) => println!("開けた?"),
        Err(e) if e.kind() == ErrorKind::NotFound => println!("見つからない: {e}"),
        Err(e) => println!("その他のエラー: {e}"),
    }
}
```

## コマンドライン引数: std::env::args

`std::env::args()` は引数のイテレータを返します。**最初の要素はプログラム名** です。

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();
    println!("プログラム名: {}", args[0]);
    println!("引数の数: {}", args.len() - 1);
    for (i, a) in args.iter().enumerate().skip(1) {
        println!("  [{i}] {a}");
    }
}
```

`cargo run -- -n 3 file.txt` のように `--` の後ろに書いた引数がプログラムへ渡ります。

実用的な CLI では、引数解析をロジックとして切り出して **`&[String]` (または `Iterator<Item=String>`)
を受け取る関数** にしておくと、テストが簡単になります。

```rust
#[derive(Debug, PartialEq)]
struct Config {
    lines: usize,
    path: Option<String>,
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let mut cfg = Config { lines: 10, path: None };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        match a.as_str() {
            "-n" => {
                let v = it.next().ok_or("-n には数値が必要です")?;
                cfg.lines = v.parse().map_err(|_| format!("数値ではありません: {v}"))?;
            }
            s if s.starts_with('-') => return Err(format!("不明なオプション: {s}")),
            s => cfg.path = Some(s.to_string()),
        }
    }
    Ok(cfg)
}

fn main() {
    let args: Vec<String> = ["-n", "3", "a.txt"].iter().map(|s| s.to_string()).collect();
    println!("{:?}", parse_args(&args));
    let bad: Vec<String> = vec!["-x".to_string()];
    println!("{:?}", parse_args(&bad));
}
```

なお、引数が UTF-8 でない可能性まで考えるなら `std::env::args_os()` (`OsString`) を使います。
`args()` は不正な UTF-8 があるとパニックします。

## 環境変数

`std::env::var(name)` は `Result<String, VarError>` を返します。設定されていなければ `Err` です。

```rust
fn main() {
    match std::env::var("HOME") {
        Ok(home) => println!("HOME = {home}"),
        Err(_) => println!("HOME は未設定"),
    }
    // 未設定ならデフォルト値
    let level = std::env::var("MY_APP_LOG").unwrap_or_else(|_| "info".to_string());
    println!("log level = {level}");

    // NO_COLOR が設定されていたら色を出さない、というのは CLI の慣習
    let color = std::env::var_os("NO_COLOR").is_none();
    println!("color enabled = {color}");
}
```

ターミナル関連でよく見る環境変数:

- `TERM` … 端末の種類 (`xterm-256color` など)
- `COLORTERM` … `truecolor` なら 24bit 色に対応
- `NO_COLOR` … 設定されていたら色を出さない (https://no-color.org)
- `COLUMNS` / `LINES` … 端末サイズ (シェルが設定する場合がある。正確な値は ioctl で取る: 第18章)

## 終了コード

プロセスは終了時に 0〜255 の数値 (終了ステータス) を返します。**0 が成功、それ以外が失敗** です。
シェルでは `echo $?` で確認でき、`cmd && next` は `cmd` が 0 のときだけ `next` を実行します。

Rust では 3 つの方法があります。

1. `main` が `Result<(), E>` を返す … `Err` なら 1 で終了し、エラーを stderr に `Debug` 表示
2. `main` が `std::process::ExitCode` を返す … 任意のコードを返せる (推奨)
3. `std::process::exit(code)` … その場で即終了 (**Drop が走らない** ので注意)

```rust
use std::process::ExitCode;

fn run() -> Result<(), String> {
    let n: i32 = "42".parse().map_err(|e| format!("parse error: {e}"))?;
    println!("n = {n}");
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
```

`run()` に本体を書き、`main` は「エラーを表示して終了コードに変換する」だけにする形は、
CLI の定番パターンです。

`process::exit` が Drop を走らせないのは TUI では重大です。raw モードを解除する Drop ガードを
置いていても、`exit` で抜けるとターミナルが壊れたまま残ります。**TUI では `exit` を使わず、
`main` から return して終わる** ようにしましょう。

## パイプとリダイレクト、isatty

```sh
cat data.txt | mytool | sort > result.txt
```

このとき `mytool` の stdin はパイプ、stdout もパイプで、端末 (tty) にはつながっていません。
良い CLI ツールは、出力先が端末かどうかで挙動を変えます。

- 端末なら色を付ける・進捗バーを出す
- パイプ/ファイルなら色なし (ANSI エスケープが混ざると後段の `grep` などが困る)

この判定が **isatty** です。C では `isatty(fd)`、Rust 標準では `std::io::IsTerminal` トレイトがあります。

```rust
use std::io::{self, IsTerminal};

fn main() {
    let tty = io::stdout().is_terminal();
    if tty {
        println!("\x1b[32m端末に出力中 (色つき)\x1b[0m");
    } else {
        println!("パイプ/ファイルに出力中 (色なし)");
    }
    eprintln!("stdin is terminal: {}", io::stdin().is_terminal());
}
```

TUI アプリは「stdin も stdout も端末」であることが前提です。起動時に `is_terminal()` を確認して、
違えばエラーメッセージを出して終了するのが親切です。

また、パイプの相手が先に終了すると (例: `mytool | head -1`)、書き込みが
`ErrorKind::BrokenPipe` で失敗します。`println!` はこのときパニックするので、大量出力する
ツールでは `writeln!` の `Result` を見て、`BrokenPipe` なら静かに終了するのが行儀のよい作法です。

## よくあるコンパイルエラーと読み方

**`error[E0599]: no method named `lines` found for struct `Stdin``** (古い書き方の場合)
→ `BufRead` トレイトがスコープにありません。`use std::io::BufRead;` を追加します。
トレイトのメソッドは `use` しないと呼べません。

**`error[E0599]: no method named `write_all` found`** / **`cannot find macro `writeln``** 系
→ `use std::io::Write;` を忘れています。

**`error[E0277]: the `?` operator can only be used in a function that returns `Result``**
→ `main` が `()` を返しているのに `?` を使っています。`fn main() -> io::Result<()>` にするか、
`run()` 関数に切り出します。

**`error[E0596]: cannot borrow `out` as mutable`**
→ `Write` のメソッドは `&mut self` です。`let mut out = ...` にします。

**`error[E0716]: temporary value dropped while borrowed`**
→ `let lock = io::stdout().lock();` は Rust 2021 以前では一時値の問題が出ることがありました。
いまは `Stdout::lock` が `'static` なロックを返すので通りますが、同様のエラーが出たら
`let stdout = io::stdout();` と変数に束縛してから `lock()` しましょう。

## まとめ

- 結果は stdout、エラーやログは stderr
- `Write` / `BufRead` をジェネリクスで受け取ると、テストで `Vec<u8>` や `&[u8]` を渡せる
- 大量出力は `lock()` + `BufWriter`、最後に `flush()`
- 引数は `env::args()` (先頭はプログラム名)、解析は純粋関数に切り出す
- 終了コードは `ExitCode`。`process::exit` は Drop を走らせない
- `IsTerminal` で端末かパイプかを判定して挙動を変える

## 次へ

第16章では、文字列を画面に表示するときの「幅」を扱います。`"あ".len()` は 3 ですが、
画面上では 2 マス分を占めます。TUI でカーソル位置を正しく計算するための必須知識です。
