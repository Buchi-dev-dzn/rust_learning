# 第19章 crossterm: イベント・代替スクリーン・後片付け

## この章のゴール

- crossterm で raw モード・代替スクリーン・カーソル制御を行える
- `execute!` と `queue!` の違いを理解し、描画をまとめて flush できる
- `event::read` / `event::poll` でキー・リサイズ・マウスのイベントを受け取れる
- `KeyEvent` (code / modifiers / kind) を正しく判定できる
- パニックしても端末を元に戻す「パニックフック + Drop ガード」を書ける

第17・18章で手書きした ANSI と termios を、crossterm はクロスプラットフォーム (macOS / Linux / Windows)
で提供してくれます。自作した知識があるので、crossterm が何をしているかが見通せるはずです。

> この章のコードは crossterm 0.29 を使うので、実行ボタンはありません (`rust,noplayground`)。
> 実際に動かすのはプロジェクト p04 です。演習は標準ライブラリだけで「crossterm 風」の設計を練習します。

## 依存関係

ワークスペースの `Cargo.toml` で `crossterm = "0.29"` が定義されているので、各プロジェクトでは

```toml
[dependencies]
crossterm = { workspace = true }
```

と書きます。

## コマンドと execute! / queue!

crossterm では、端末への操作を **コマンド** (型) で表します。

| モジュール | コマンド | 対応する ANSI (第17章) |
|---|---|---|
| `cursor` | `MoveTo(x, y)` | `ESC [ y+1 ; x+1 H` |
| `cursor` | `Hide` / `Show` | `ESC [ ? 25 l` / `h` |
| `terminal` | `Clear(ClearType::All)` | `ESC [ 2 J` |
| `terminal` | `EnterAlternateScreen` / `LeaveAlternateScreen` | `ESC [ ? 1049 h` / `l` |
| `style` | `SetForegroundColor(Color::Red)` | `ESC [ 31 m` など |
| `style` | `Print("text")` | そのまま書く |
| `style` | `ResetColor` | `ESC [ 0 m` 相当 |

crossterm の座標は **0 始まりの (列, 行) = (x, y)** です。ANSI の 1 始まり `行;列` への変換は
crossterm がやってくれます。

コマンドを実行するマクロは 2 つあります。

- `execute!(w, cmd1, cmd2, ...)` … 書いて **すぐ flush**
- `queue!(w, cmd1, cmd2, ...)` … バッファに書くだけ。あとで `w.flush()` する

```rust,noplayground
use std::io::{self, Write};
use crossterm::{
    cursor, execute, queue,
    style::{self, Color, Stylize},
    terminal::{self, ClearType},
};

fn main() -> io::Result<()> {
    let mut out = io::stdout();

    // 1 つだけならすぐ反映でよい
    execute!(out, terminal::Clear(ClearType::All))?;

    // 描画はまとめて queue して最後に 1 回 flush (ちらつき防止)
    for y in 0..5 {
        queue!(
            out,
            cursor::MoveTo(y * 2, y),
            style::SetForegroundColor(Color::Cyan),
            style::Print(format!("line {y}")),
            style::ResetColor,
        )?;
    }
    // Stylize トレイトで文字列に直接スタイルを付けることもできる
    queue!(out, cursor::MoveTo(0, 6), style::PrintStyledContent("bold red".red().bold()))?;
    out.flush()?;
    Ok(())
}
```

1 フレームの描画は `queue!` で貯めて最後に `flush` が基本です。第15章の `BufWriter` と同じ考え方です。

## raw モードと代替スクリーン

```rust,noplayground
use crossterm::{cursor, execute, terminal};
use std::io;

fn setup() -> io::Result<()> {
    terminal::enable_raw_mode()?; // termios を raw に (Windows ではコンソールモードを変更)
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    Ok(())
}

fn restore() -> io::Result<()> {
    execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;
    Ok(())
}
```

`enable_raw_mode` は第18章の `tcgetattr` → `cfmakeraw` → `tcsetattr` を、`disable_raw_mode` は
保存しておいた設定の復元を、内部でやっています。

端末サイズは `terminal::size()` で `(列数, 行数)` が取れます。

```rust,noplayground
let (cols, rows) = crossterm::terminal::size()?;
```

## イベントを読む

```rust,noplayground
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use std::time::Duration;

fn event_loop() -> std::io::Result<()> {
    loop {
        // 最大 100ms 待つ。イベントがなければ false
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if handle_key(key) {
                        break;
                    }
                }
                Event::Resize(cols, rows) => {
                    // 新しいサイズで描き直す
                    let _ = (cols, rows);
                }
                Event::Mouse(_) | Event::FocusGained | Event::FocusLost | Event::Paste(_) => {}
                _ => {}
            }
        } else {
            // タイムアウト: アニメーションや時計の更新などをここで
        }
    }
    Ok(())
}

/// true を返したら終了
fn handle_key(key: KeyEvent) -> bool {
    match (key.code, key.modifiers) {
        (KeyCode::Char('q'), _) | (KeyCode::Esc, _) => true,
        (KeyCode::Char('c'), m) if m.contains(KeyModifiers::CONTROL) => true,
        (KeyCode::Up, _) | (KeyCode::Char('k'), KeyModifiers::NONE) => false, // 上へ
        _ => false,
    }
}
```

ポイント:

- `event::read()` はイベントが来るまで **ブロック** する。何かを定期的に更新したいなら
  `event::poll(timeout)` で「来ているか」を確認してから `read()` する
- `KeyEvent` は `code` (`KeyCode::Char('a')`, `KeyCode::Enter`, `KeyCode::Up`, `KeyCode::F(1)` ...)、
  `modifiers` (`KeyModifiers::CONTROL | SHIFT | ALT` のビットフラグ)、`kind` (`Press` / `Repeat` / `Release`)、
  `state` を持つ
- **`kind == KeyEventKind::Press` を確認する**。Windows ではキーを離したときにも `Release`
  イベントが届くため、確認しないと 1 回の入力が 2 回処理される
- Ctrl-C は raw モードではシグナルにならず、`KeyCode::Char('c')` + `CONTROL` として届く。
  自分で終了処理を書かないと、ユーザーがアプリを終了できなくなる
- 第18章で自作したパーサの `Key` 型は、crossterm の `KeyCode` とほぼ同じ形をしている

### マウスとペースト

マウスやブラケットペーストは明示的に有効化します。終了時には無効化を忘れずに。

```rust,noplayground
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, EnableBracketedPaste, DisableBracketedPaste};
execute!(out, EnableMouseCapture, EnableBracketedPaste)?;
// ... Event::Mouse(MouseEvent { kind, column, row, modifiers }) が届く
execute!(out, DisableBracketedPaste, DisableMouseCapture)?;
```

## 後片付け: Drop ガード + パニックフック

TUI で最も大事なのは「どんな終わり方をしても端末を元に戻す」ことです。
理由がもう 1 つあります。**パニックメッセージは代替スクリーン上に表示される** ので、
後から元の画面に戻るとメッセージが消えて見えません。しかも raw モードだと改行が崩れて読めません。

そこで、パニックフック (パニックが起きたときに最初に呼ばれる関数) で **先に端末を戻してから**
元のフック (メッセージ表示) を呼びます。

```rust,noplayground
use std::io;
use crossterm::{cursor, execute, terminal};

fn restore_terminal() {
    // 失敗しても続行したいので結果は無視する
    let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

fn install_panic_hook() {
    let original = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();   // 1. まず端末を戻す
        original(info);       // 2. 通常のパニックメッセージを表示
    }));
}

struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

fn main() -> io::Result<()> {
    install_panic_hook();
    let _guard = TerminalGuard::new()?;
    run()   // ? でエラーが返っても、パニックしても、_guard の Drop で戻る
}
```

`restore_terminal` はパニックフックと Drop の両方から呼ばれる可能性があるので、
**2 回呼ばれても問題ない (冪等)** ように書きます。`LeaveAlternateScreen` や `disable_raw_mode` は
2 回呼んでも害はありませんが、自作のバックエンドでは「もう戻したか」をフラグで管理すると安全です (演習 ex02)。

> ratatui 0.30 の `ratatui::init()` は、raw モード・代替スクリーン・パニックフックの設定を
> まとめてやってくれます (第21章)。中身はここで書いたものとほぼ同じです。

## テストしやすい設計

crossterm の `KeyEvent` をアプリの中で直接使い回すと、ロジックのテストに crossterm が必要になり、
キー割り当ての変更も大変です。おすすめは次の 2 段構えです。

1. 入力層: `crossterm::event::KeyEvent` → アプリ独自の `Action` enum に変換 (`fn map_key`)
2. ロジック層: `Action` を受け取って状態を更新 (`fn update`, 第20章)

```rust
// 標準ライブラリだけで書いた「crossterm 風」の型で、変換部分を試す
#[derive(Debug, Clone, Copy, PartialEq)]
enum KeyCode { Char(char), Up, Down, Esc }

#[derive(Debug, Clone, Copy)]
struct KeyEvent { code: KeyCode, ctrl: bool }

#[derive(Debug, PartialEq)]
enum Action { Quit, MoveUp, MoveDown }

fn map_key(k: KeyEvent) -> Option<Action> {
    match k.code {
        KeyCode::Char('c') if k.ctrl => Some(Action::Quit),
        KeyCode::Char('q') | KeyCode::Esc => Some(Action::Quit),
        KeyCode::Up | KeyCode::Char('k') => Some(Action::MoveUp),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::MoveDown),
        _ => None,
    }
}

fn main() {
    let keys = [
        KeyEvent { code: KeyCode::Char('j'), ctrl: false },
        KeyEvent { code: KeyCode::Up, ctrl: false },
        KeyEvent { code: KeyCode::Char('x'), ctrl: false },
        KeyEvent { code: KeyCode::Char('c'), ctrl: true },
    ];
    for k in keys {
        println!("{:?} -> {:?}", k, map_key(k));
    }
}
```

本番では `map_key` の引数が `crossterm::event::KeyEvent` になるだけです。
`Action` に変換してしまえば、`update` のテストは端末なしで書けます。

同じように、端末への出力も `impl Write` で受け取れば、`Vec<u8>` に書かせて
「どんな ANSI が出たか」をテストできます (`queue!` は任意の `Write` に使えます)。

## crossterm の主な型 早見表

| やりたいこと | crossterm |
|---|---|
| raw モード | `terminal::enable_raw_mode()` / `disable_raw_mode()` |
| 画面サイズ | `terminal::size() -> io::Result<(u16, u16)>` |
| 代替スクリーン | `terminal::EnterAlternateScreen` / `LeaveAlternateScreen` |
| 消去 | `terminal::Clear(ClearType::All / CurrentLine / UntilNewLine)` |
| カーソル | `cursor::MoveTo(x, y)`, `Hide`, `Show`, `SetCursorStyle::BlinkingBar` |
| 色 | `style::SetForegroundColor(Color::Rgb { r, g, b })`, `SetBackgroundColor`, `Color::AnsiValue(n)` |
| 装飾 | `style::SetAttribute(Attribute::Bold)`, `"text".bold().underlined()` |
| 入力待ち | `event::poll(Duration) -> io::Result<bool>`, `event::read() -> io::Result<Event>` |
| イベント | `Event::Key`, `Event::Mouse`, `Event::Resize(cols, rows)`, `Event::Paste(String)` |

## よくあるコンパイルエラーと読み方

**`error: cannot find macro `execute` in this scope`**
→ `use crossterm::execute;` (または `crossterm::execute!(...)`) が必要です。`queue!` も同様です。

**`error[E0599]: no method named `flush` found`**
→ `use std::io::Write;` がありません。`queue!` の後の flush に必要です。

**`error[E0277]: the trait bound `...: crossterm::Command` is not satisfied`**
→ `execute!` にコマンドではない値を渡しています。文字列は `style::Print("...")` で包みます。

**`error[E0004]: non-exhaustive patterns: `Event::FocusGained` not covered`**
→ `Event` や `KeyCode` は多くのバリアントを持ちます。使わないものは `_ => {}` で受けます。

**キーが 2 回反応する** (コンパイルは通る)
→ `KeyEventKind::Press` をチェックしていません。

**終了後にターミナルがおかしい** (コンパイルは通る)
→ `disable_raw_mode` / `LeaveAlternateScreen` が呼ばれていません。`process::exit` を使っていないか、
ガードを `let _ =` で即 Drop していないか確認しましょう。直すには端末で `reset` と打ちます。

## まとめ

- crossterm はコマンド型 + `execute!` (即 flush) / `queue!` (貯める) で端末を操作する
- 座標は 0 始まりの (x, y)
- `event::poll` + `event::read` でブロックせずにイベントを処理する。`KeyEventKind::Press` を確認
- 端末の復元は Drop ガード + パニックフックの二重で守る。復元処理は冪等に
- crossterm の型はアプリ独自の `Action` に変換し、ロジックを端末から切り離す

## 次へ

第20章では、`Action` を受け取って状態を更新し、状態から画面を描くという
**TUI アプリの全体構造** (Elm アーキテクチャ) を学びます。
