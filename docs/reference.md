# 早見表

学習中に何度も見返す情報をまとめています。詳しい説明はそれぞれの章で。

## 所有権と借用のルール

| ルール | 意味 |
|---|---|
| 値の所有者は常に 1 つ | 代入・関数呼び出しで **move** すると元の変数は使えない |
| `Copy` 型はコピーされる | 整数・浮動小数点・`bool`・`char`・`Copy` な要素だけのタプル |
| 共有参照 `&T` はいくつでも | 読み取り専用 |
| 可変参照 `&mut T` は同時に 1 つだけ | しかも共有参照と同時には存在できない |
| 参照は所有者より長生きできない | これを検査するのがライフタイム |

迷ったら: **関数の引数は `&str` / `&[T]` / `&T` で受け取り、所有が必要なときだけ `String` / `Vec<T>` / `T` を受け取る。**

## よく出るコンパイルエラー

| コード | メッセージ (抜粋) | よくある原因 → 対処 |
|---|---|---|
| E0382 | borrow of moved value | move 済みの値を使った → `&` で借りる / `.clone()` / 設計を見直す |
| E0499 | cannot borrow as mutable more than once | `&mut` を 2 つ同時に → スコープを分ける、インデックスで操作 |
| E0502 | cannot borrow as mutable because it is also borrowed as immutable | `for x in &v { v.push(..) }` など → 先に集めてから変更 |
| E0505 | cannot move out because it is borrowed | 借用中に move → 借用を先に終わらせる |
| E0106 | missing lifetime specifier | 参照を返す関数 → `fn f<'a>(x: &'a str) -> &'a str` |
| E0308 | mismatched types | 型違い → `&String` と `&str`、`i32` と `usize` などを確認。`as` / `.into()` |
| E0277 | the trait bound ... is not satisfied | トレイト未実装 → `#[derive(Debug, Clone, PartialEq)]` / `impl` を書く |
| E0599 | no method named ... found | トレイトを `use` していない (`use std::io::Write;` など) |
| E0384 | cannot assign twice to immutable variable | `let` → `let mut` |
| E0425 | cannot find value | スペルミス / スコープ外 / `use` 忘れ |
| E0433 | failed to resolve | モジュールパスの誤り → `crate::` / `super::` |

エラーの詳しい説明: ターミナルで `rustc --explain E0382`

## 変換チートシート

```rust
fn main() {
    let s: String = "abc".to_string();   // &str → String
    let r: &str = &s;                    // String → &str
    let n: i32 = "42".parse().unwrap();  // &str → 数値
    let t: String = n.to_string();       // 数値 → String
    let v: Vec<char> = s.chars().collect();
    let back: String = v.iter().collect();
    let bytes: &[u8] = s.as_bytes();
    let u: usize = n as usize;           // 数値同士は as (切り捨てに注意)
    println!("{s} {r} {n} {t} {back} {bytes:?} {u}");
}
```

## Option / Result の便利メソッド

| メソッド | 使いどころ |
|---|---|
| `?` | エラー/None を呼び出し元へ伝播 |
| `unwrap_or(v)` / `unwrap_or_default()` | デフォルト値 |
| `map(f)` / `and_then(f)` | 中身を変換 / 連鎖 |
| `ok_or(e)` / `ok_or_else(f)` | `Option` → `Result` |
| `.ok()` | `Result` → `Option` |
| `if let Some(x) = ...` / `let Some(x) = ... else { return }` | 分岐 |

## イテレータ

| アダプタ | 例 |
|---|---|
| `map` / `filter` / `filter_map` | `lines.filter(|l| !l.is_empty())` |
| `enumerate` / `zip` / `rev` / `skip` / `take` | 行番号付き表示、スクロール |
| `chain` / `flat_map` / `windows(n)` / `chunks(n)` | 連結、分割 |
| `collect::<Vec<_>>()` / `sum` / `count` / `max_by_key` / `position` / `any` / `all` | 終端 |

## ANSI エスケープシーケンス

`ESC` = `\x1b`、`CSI` = `\x1b[`。座標は **1 始まり** (行;列)。

| シーケンス | 意味 |
|---|---|
| `\x1b[{row};{col}H` | カーソル移動 |
| `\x1b[{n}A` / `B` / `C` / `D` | 上 / 下 / 右 / 左へ n |
| `\x1b[2J` | 画面全消去 |
| `\x1b[K` | カーソルから行末まで消去 |
| `\x1b[?25l` / `\x1b[?25h` | カーソル非表示 / 表示 |
| `\x1b[?1049h` / `\x1b[?1049l` | 代替スクリーンに入る / 出る |
| `\x1b[0m` | 装飾リセット |
| `\x1b[1m` / `2m` / `3m` / `4m` / `7m` | 太字 / 薄く / 斜体 / 下線 / 反転 |
| `\x1b[30m`〜`37m` / `90m`〜`97m` | 前景色 (標準 / 明るい) |
| `\x1b[40m`〜`47m` / `100m`〜`107m` | 背景色 |
| `\x1b[38;5;{n}m` / `\x1b[48;5;{n}m` | 256 色 |
| `\x1b[38;2;{r};{g};{b}m` | 24bit カラー |
| `\x1b[6n` | カーソル位置の問い合わせ (応答 `\x1b[{row};{col}R`) |
| `\x1b[?2004h` | bracketed paste 有効化 |

```rust
fn main() {
    for (i, name) in ["black", "red", "green", "yellow", "blue", "magenta", "cyan", "white"].iter().enumerate() {
        print!("\x1b[3{i}m{name}\x1b[0m ");
    }
    println!();
    println!("\x1b[1;4m太字+下線\x1b[0m \x1b[38;2;244;102;35mRust オレンジ\x1b[0m");
}
```

## キー入力のバイト列 (raw モード, xterm 系)

| キー | バイト列 |
|---|---|
| 通常文字 | UTF-8 そのまま (`あ` = `E3 81 82`) |
| Enter / Tab / Backspace | `\r` (0x0D) / `\t` (0x09) / 0x7F |
| Ctrl + A〜Z | 0x01〜0x1A (`Ctrl-C` = 0x03, `Ctrl-Q` = 0x11) |
| Esc | 0x1B 単独 (後続がない) |
| ↑ ↓ → ← | `\x1b[A` `\x1b[B` `\x1b[C` `\x1b[D` (アプリケーションモードでは `\x1bOA`…) |
| Home / End | `\x1b[H` `\x1b[F` (または `\x1b[1~` `\x1b[4~`) |
| Insert / Delete / PgUp / PgDn | `\x1b[2~` `\x1b[3~` `\x1b[5~` `\x1b[6~` |
| F1〜F4 | `\x1bOP` `\x1bOQ` `\x1bOR` `\x1bOS` |
| Alt + x | `\x1b` + `x` |

## termios (raw モード) の要点

```rust,noplayground
let mut t: libc::termios = unsafe { std::mem::zeroed() };
unsafe { libc::tcgetattr(libc::STDIN_FILENO, &mut t) };   // 元の設定を保存しておく
let orig = t;
unsafe { libc::cfmakeraw(&mut t) };                       // ICANON, ECHO, ISIG などを外す
t.c_cc[libc::VMIN] = 0;  t.c_cc[libc::VTIME] = 1;         // read を 0.1 秒でタイムアウト
unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &t) };
// ... 終了時 (Drop) に必ず orig を tcsetattr で戻す
```

## crossterm 0.29

```rust,noplayground
use crossterm::{cursor, event::{self, Event, KeyCode, KeyEventKind}, execute, queue, style, terminal};
use std::io::Write;

terminal::enable_raw_mode()?;
execute!(out, terminal::EnterAlternateScreen, cursor::Hide)?;
queue!(out, terminal::Clear(terminal::ClearType::All), cursor::MoveTo(0, 0),
       style::SetForegroundColor(style::Color::Green), style::Print("hello"), style::ResetColor)?;
out.flush()?;
if event::poll(std::time::Duration::from_millis(100))? {
    match event::read()? {
        Event::Key(k) if k.kind == KeyEventKind::Press && k.code == KeyCode::Char('q') => { /* 終了 */ }
        Event::Resize(w, h) => { /* 再描画 */ }
        _ => {}
    }
}
execute!(out, cursor::Show, terminal::LeaveAlternateScreen)?;
terminal::disable_raw_mode()?;
```

## ratatui 0.30

```rust,noplayground
use ratatui::{layout::{Constraint, Layout}, widgets::{Block, List, ListState, Paragraph}, Frame};

let mut terminal = ratatui::init();          // raw モード + 代替画面 + パニックフック
let mut state = ListState::default().with_selected(Some(0));
loop {
    terminal.draw(|f: &mut Frame| {
        let [top, main] = Layout::vertical([Constraint::Length(3), Constraint::Fill(1)]).areas(f.area());
        f.render_widget(Paragraph::new("ヘッダー").block(Block::bordered()), top);
        f.render_stateful_widget(List::new(["a", "b"]).highlight_symbol("> "), main, &mut state);
    })?;
    // event::read() でキー処理 … break で抜ける
}
ratatui::restore();
```

## TUI アプリの基本形

```text
初期化 (raw モード, 代替画面, パニックフック)
loop {
    draw(&app)            // 状態 → 画面 (純粋な関数に近づける)
    event = 入力 or tick   // poll(timeout) で待つ
    app.update(event)     // 状態遷移 (ここを単体テストする)
    if app.should_quit { break }
}
後片付け (必ず! Drop ガードで)
```
