# crossterm 入門: イベントループ

p03 では libc と ANSI を手で書いてターミナルを操作しました。今度は同じことを **crossterm** で行います。
crossterm は raw モード・代替スクリーン・キー解析・リサイズ検出を、OS の違いを吸収して提供してくれるライブラリです。
カウンタ・キーログ・時計 (ティック) を持つ小さなアプリを作りながら、TUI の心臓部である **イベントループ** を組み立てます。

## 完成イメージ

```text
 crossterm basics                       (太字 + 反転)

  カウンタ   : 3
  経過時間   : 12.25 秒
  ティック数 : 49
  端末サイズ : 80 x 24

  最近のキー:
    +
    Up
    Ctrl+a

  +/k/Up: 増やす  -/j/Down: 減らす  r: リセット  Space: 一時停止  !: パニック  q/Esc: 終了
```

- キーを押さなくても 0.25 秒ごとに経過時間が進む (ティック)。Space で一時停止。
- 端末の大きさを変えると「端末サイズ」が即座に変わる。
- `q` / `Esc` / `Ctrl-C` で終了。`!` でわざとパニックしても、端末は元に戻りメッセージが読める。

## 学ぶこと

- `crossterm::terminal` の raw モード / 代替スクリーン、`cursor::Hide`、`execute!` と `queue!` の違い
- `event::read` (ブロッキング) と `event::poll` (タイムアウト付き) によるイベントループ + ティック
- `Event::Key` / `KeyEvent` / `KeyModifiers` / `KeyEventKind`、`Event::Resize`
- Drop ガード + パニックフックによる後片付け (p03 と同じ考え方)
- 状態 (`App`)・メッセージ (`Msg`)・更新 (`update`)・表示 (`view`) の分離と、そのテスト

作業は `projects/p04_crossterm_basics/starter/` で行います。
テストは `cargo test -p p04_crossterm_basics_starter`、実行は `cargo run -p p04_crossterm_basics_starter` (学習サイトでは端末ボタンから)。

## STEP 1: 端末の準備と後片付け (TerminalGuard)

やること
- `main.rs` の `TerminalGuard::enter` と `restore_terminal` を実装する。
- `draw` で "Hello" を表示してみる。

コード例
```rust,noplayground
fn enter() -> io::Result<TerminalGuard> {
    terminal::enable_raw_mode()?;
    let guard = TerminalGuard; // 次の行で失敗しても Drop で raw モードを戻せるよう先に作る
    execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
    Ok(guard)
}

fn restore_terminal() {
    let _ = execute!(io::stdout(), cursor::Show, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}
```

```rust,noplayground
queue!(out, terminal::Clear(ClearType::All), cursor::MoveTo(0, 0), Print("Hello"))?;
out.flush()
```

ポイント
- `enable_raw_mode` は p03 で書いた termios の操作そのものです。`EnterAlternateScreen` は `ESC [ ? 1049 h` を出力します。
- **`execute!` はすぐに flush、`queue!` は溜めるだけ** です。描画では `queue!` でまとめて送り、最後に 1 回 `flush` すると
  ちらつきが減ります。
- `restore_terminal` の中ではエラーを `let _ =` で捨てています。後片付け中に失敗しても、できることはないからです。

確認方法
- `cargo build -p p04_crossterm_basics_starter` が通る。
- 実行すると "Hello" が表示された直後に `todo!("event loop")` でパニックする。
  **パニックメッセージは代替スクリーンに書かれて消えてしまう** はずです (次の STEP で直します)。
  `reset` と打てば端末は戻ります。

## STEP 2: パニックフック

やること
- `install_panic_hook` を実装する。
- `lib.rs` の `handle_key` に、`'!'` でわざとパニックする分岐を追加する (後で動作確認に使う)。

コード例
```rust,noplayground
fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));
}
```

ポイント
- パニック時は「フック → unwind (Drop)」の順に動きます。フックで先に端末を戻すと、標準のパニック表示が普通の画面に出ます。
  その後 Drop でもう一度 `restore_terminal` が呼ばれますが、無害です。
- `take_hook` で取り出した標準のフックを `move` クロージャに閉じ込めて呼ぶ、という形はよく使うので覚えておきましょう。

確認方法
- 実行すると、今度は `todo!("event loop")` のパニックメッセージが **元の画面に** 表示され、端末も正常に戻る。

## STEP 3: イベントを読む (event::read)

やること
- `run` のループで `event::read()` を呼び、`Event::Key` を `Msg::Key` にして `app.update` に渡す。

コード例
```rust,noplayground
while !app.quit {
    draw(&mut out, &app)?;
    if let Event::Key(key) = event::read()? {
        app.update(Msg::Key(key));
    }
}
```

ポイント
- `event::read` はイベントが来るまで **ブロック** します。キーを押さないと画面は一切更新されません。
- `KeyEvent` は `code` (`KeyCode::Char('a')`, `KeyCode::Up` ...) と `modifiers` (`KeyModifiers::CONTROL` ...) を持ちます。
  p03 で自作したバイト列パーサの仕事を、crossterm が代わりにやってくれています。
- `handle_key` には q / Esc / Ctrl-C で `quit = true` にする処理がすでに書かれています。

確認方法
- 実行して `q` で終了でき、端末が元に戻る。`!` を押すとパニックメッセージが表示されて終了する。

## STEP 4: タイムアウト付きで待つ (event::poll とティック)

やること
- `event::read` の前に `event::poll(timeout)` を挟み、「次のティックまでの残り時間」だけ待つ。
- 時間が来たら `Msg::Tick` を送る。`lib.rs` の `Msg::Tick` で `ticks` を増やす (一時停止中は増やさない)。
- 再描画は「状態が変わったとき (`update` が true を返したとき)」だけにする。

コード例
```rust,noplayground
let mut last_tick = Instant::now();
let mut dirty = true;
while !app.quit {
    if dirty {
        draw(&mut out, &app)?;
        dirty = false;
    }
    let timeout = TICK_RATE.saturating_sub(last_tick.elapsed());
    if event::poll(timeout)? {
        if let Event::Key(key) = event::read()? {
            dirty |= app.update(Msg::Key(key));
        }
    }
    if last_tick.elapsed() >= TICK_RATE {
        dirty |= app.update(Msg::Tick);
        last_tick = Instant::now();
    }
}
```

```rust,noplayground
Msg::Tick => {
    if self.paused {
        false
    } else {
        self.ticks += 1;
        true
    }
}
```

ポイント
- `poll(timeout)` は「timeout までにイベントが来れば true」。true のときだけ `read` すればブロックしません。
- 固定の `poll(250ms)` ではなく **残り時間** を渡すのがコツです。キーを連打しても、ティックの間隔が乱れません。
- これがほぼすべての TUI アプリに共通するイベントループの形です (アニメーション、時計、バックグラウンド処理の結果表示 ...)。

確認方法
- `cargo test -p p04_crossterm_basics_starter tick` は STEP 6 の Space 処理が無いとまだ失敗します。ここでは実行して確認:
  STEP 7 の表示がまだなので、`draw` に `Print(format!("ticks: {}", app.ticks))` を一時的に加えると、何もしなくても数字が増えていく。

## STEP 5: リサイズに対応する

やること
- `Event::Resize(cols, rows)` を `Msg::Resize` に変換して渡す。
- `lib.rs` の `Msg::Resize` で `self.size` を更新して `true` を返す。

コード例
```rust,noplayground
if event::poll(timeout)? {
    let msg = match event::read()? {
        Event::Key(key) => Some(Msg::Key(key)),
        Event::Resize(c, r) => Some(Msg::Resize(c, r)),
        _ => None, // マウスやフォーカスなどは今回は無視
    };
    if let Some(msg) = msg {
        dirty |= app.update(msg);
    }
}
```

ポイント
- p03 では毎回 ioctl でサイズを問い合わせました (ポーリング)。crossterm は内部で SIGWINCH シグナルを受け取り、
  イベントとして届けてくれます。
- 端末サイズは状態の一部です。レイアウト計算 (何行表示できるか等) は `App` が持つ `size` を使います。

確認方法
- `cargo test -p p04_crossterm_basics_starter resize` が通る。

## STEP 6: App の更新ロジックを完成させる

やること
- `lib.rs` の `update` で `KeyEventKind::Press` 以外を無視し、`log_key` を呼ぶ。
- `handle_key` にカウンタ (+ / k / Up, - / j / Down)、`r` (リセット)、Space (一時停止) を追加する。
- `log_key` と `format_key` を実装する。

コード例
```rust,noplayground
Msg::Key(key) => {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    self.log_key(&key);
    self.handle_key(key);
    true
}
```

```rust,noplayground
fn log_key(&mut self, key: &KeyEvent) {
    if self.key_log.len() == KEY_LOG_LEN {
        self.key_log.pop_front();
    }
    self.key_log.push_back(format_key(key));
}

pub fn format_key(key: &KeyEvent) -> String {
    let mut s = String::new();
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        s.push_str("Ctrl+");
    }
    // ALT, SHIFT (文字以外のとき) も同様に
    match key.code {
        KeyCode::Char(' ') => s.push_str("Space"),
        KeyCode::Char(c) => s.push(c),
        KeyCode::F(n) => s.push_str(&format!("F{n}")),
        KeyCode::Up => s.push_str("Up"),
        // ...
        other => s.push_str(&format!("{other:?}")),
    }
    s
}
```

ポイント
- Windows ではキーを離したとき (`Release`) にもイベントが来ます。`Press` だけを扱うのが移植性のある書き方です。
- `VecDeque` は先頭からも末尾からも O(1) で出し入れできるので、「最新 N 件のログ」にぴったりです。
- `App` は crossterm の型 (`KeyEvent`) を受け取りますが、端末には触れません。だから `KeyEvent::new(...)` を作って
  テストに渡せます。テストコードを読んで、どんな入力でどう変わるべきかを確認しましょう。

確認方法
- `cargo test -p p04_crossterm_basics_starter` がすべて通る。

## STEP 7: view を描画する

やること
- `draw` を完成させる: 画面を消去し、`app.view()` の各行を `MoveTo(0, i)` + `Print` で書く。1 行目は太字 + 反転。
  端末の行数より多い行は書かない。

コード例
```rust,noplayground
fn draw(out: &mut Stdout, app: &App) -> io::Result<()> {
    queue!(out, terminal::Clear(ClearType::All))?;
    let rows = app.size.1 as usize;
    for (i, line) in app.view().iter().enumerate().take(rows) {
        queue!(out, cursor::MoveTo(0, i as u16))?;
        if i == 0 {
            queue!(
                out,
                SetAttribute(Attribute::Bold),
                SetAttribute(Attribute::Reverse),
                Print(format!(" {line} ")),
                SetAttribute(Attribute::Reset)
            )?;
        } else {
            queue!(out, Print(line))?;
        }
    }
    out.flush()
}
```

ポイント
- raw モードでは `\n` で行頭に戻らないので、行ごとに `MoveTo` します (p03 と同じ)。
- `view` は「状態 → 表示する行」の純粋な関数、`draw` は「行 → 端末」の変換だけ。この分担が次の p05 (エディタ)、
  p06 (ratatui) でも続きます。ratatui ではこの `draw` の部分をライブラリが担当してくれます。

確認方法
- `cargo run -p p04_crossterm_basics_starter` で完成イメージどおりに表示される。
  - 何もしなくても経過時間が進む / Space で止まる
  - `+` `-` `↑` `↓` でカウンタが変わり、キーログに記録される
  - 端末サイズを変えると表示が追従する
  - `q` で終了して元の画面に戻る。`!` でパニックさせてもメッセージが読め、端末が壊れない
- 完成版 `cargo run -p p04_crossterm_basics` と同じ動きになれば完成です。

## 発展課題

- `crossterm::event::EnableMouseCapture` を有効にし、クリックした座標をキーログに出す。
- 経過時間を `mm:ss` 形式にし、カウンタの値に応じて色を変える (`style::Stylize` の `.red()` など)。
- 全消去 (`Clear(All)`) をやめ、行ごとに `Clear(UntilNewLine)` で上書きしてちらつきをなくす。
- 別スレッドで重い処理 (例: 1 秒ごとに何かを計算) を走らせ、`std::sync::mpsc` で結果をメインループに送って `Msg` として処理する。
