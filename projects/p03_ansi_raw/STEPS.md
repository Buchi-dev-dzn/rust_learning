# ansi_raw: ライブラリなしでターミナル制御

crossterm などのライブラリを使わず、**ANSI エスケープシーケンスと libc の termios だけ** でターミナルを操ります。
矢印キーで箱を動かせる小さなデモを作りながら、TUI ライブラリが裏で何をしているのかを自分の手で確かめます。

## 完成イメージ

```text
 ansi_raw demo - arrows/hjkl: move  Home/End/PgUp/PgDn  q: quit



                                  ┌──────────┐
                                  │          │
                                  │  Rust!   │
                                  │          │
                                  └──────────┘


 key: Up  bytes: [1b 5b 41]  pos: 34,8  size: 80x24          (反転表示)
```

- 矢印キー / `hjkl` で箱が動く。Home/End/PageUp/PageDown で端まで移動。
- 最下行に「押したキー」と「端末から届いた生のバイト列」が表示される。日本語を入力すると UTF-8 のバイト列が見える。
- 端末のサイズを変えると追従する。
- `q` (または Ctrl-C / Ctrl-Q) で終了すると、元のシェル画面がそのまま戻る。パニックしても戻る。

## 学ぶこと

- ANSI エスケープシーケンス (カーソル移動・色・画面消去・代替スクリーン・カーソル非表示)
- 端末の「cooked モード」と「raw モード」、libc の `termios` (`tcgetattr` / `tcsetattr`)
- RAII: `Drop` ガードで後片付けを保証する / パニックフック
- `ioctl(TIOCGWINSZ)` で端末サイズを得る
- `unsafe` と FFI の最小限の使い方 (`MaybeUninit`, `// SAFETY:` コメント)
- キー入力のバイト列 (制御文字・エスケープシーケンス・UTF-8) を自分でパースする

作業は `projects/p03_ansi_raw/starter/` で行います。
テストは `cargo test -p p03_ansi_raw_starter`、実行は `cargo run -p p03_ansi_raw_starter` (学習サイトでは端末ボタンから)。

> もし端末がおかしくなったら (入力が見えない、改行がずれる)、何も見えなくても `reset` と打って Enter を押せば戻ります。

## STEP 1: ANSI エスケープシーケンスで描く

やること
- `starter/src/ansi.rs` の `move_to`, `fg`, `bg` を実装する。
- 定数 (`CLEAR_SCREEN`, `HIDE_CURSOR`, `ENTER_ALT_SCREEN` ...) の意味を読んでおく。

コード例
```rust,noplayground
pub fn move_to(row: u16, col: u16) -> String {
    format!("\x1b[{};{}H", row + 1, col + 1)
}

pub fn fg(color: u8) -> String {
    format!("\x1b[38;5;{color}m")
}
```

ポイント
- 端末に送る文字列のうち `ESC [` (= `\x1b[`) で始まる部分は表示されず、命令として解釈されます (CSI シーケンス)。
  `H` はカーソル移動、`J` は消去、`m` は文字装飾 (SGR) です。
- ANSI の座標は **1 始まり**。プログラム内は 0 始まりで扱い、出力するときに +1 するのが混乱しないコツです。
- 試しに、普通の Rust プログラムで次を実行してみましょう (標準ライブラリだけで動きます)。

```rust
fn main() {
    print!("\x1b[2J");                 // 画面消去
    print!("\x1b[5;10H");              // 5 行目 10 列目へ
    print!("\x1b[38;5;45mHello\x1b[0m"); // 水色で Hello、装飾解除
    println!("\x1b[10;1H");            // 10 行目へ移動して改行
}
```

確認方法
- `cargo test -p p03_ansi_raw_starter step1` が通る。

## STEP 2: raw モードに入る (termios) と Drop ガード

やること
- `terminal.rs` の `RawMode::enable` と `restore` を実装する。
  1. `tcgetattr` で現在の設定を取得し、`ORIGINAL` に保存
  2. コピーを raw 用に書き換えて `tcsetattr`
  3. `restore` で `ORIGINAL` を `take()` して `tcsetattr` で戻す (Drop から呼ばれる)

コード例
```rust,noplayground
let fd = libc::STDIN_FILENO;
let mut orig = MaybeUninit::<libc::termios>::uninit();
// SAFETY: tcgetattr は成功すると orig を初期化する
if unsafe { libc::tcgetattr(fd, orig.as_mut_ptr()) } != 0 {
    return Err(io::Error::last_os_error());
}
let orig = unsafe { orig.assume_init() };

let mut raw = orig;
raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
raw.c_oflag &= !libc::OPOST;
raw.c_cflag |= libc::CS8;
raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
raw.c_cc[libc::VMIN] = 0;   // 1 バイトも来なくても
raw.c_cc[libc::VTIME] = 1;  // 0.1 秒で read が戻る

*ORIGINAL.lock().unwrap() = Some(orig);
if unsafe { libc::tcsetattr(fd, libc::TCSAFLUSH, &raw) } != 0 {
    return Err(io::Error::last_os_error());
}
```

```rust,noplayground
pub fn restore() {
    let orig = match ORIGINAL.lock() {
        Ok(mut g) => g.take(),
        Err(poisoned) => poisoned.into_inner().take(),
    };
    if let Some(orig) = orig {
        unsafe { libc::tcsetattr(libc::STDIN_FILENO, libc::TCSAFLUSH, &orig) };
    }
}
```

ポイント
- 通常 (cooked モード) の端末は、Enter が押されるまで入力を溜め (ICANON)、打った文字を表示し (ECHO)、
  Ctrl-C で SIGINT を送ります (ISIG)。エディタやゲームでは 1 キーごとに反応したいので、これらを全部切ります。
- `OPOST` を切ると `\n` が `\r\n` に変換されなくなります。raw モード中に `println!` すると階段状にずれるのはこのためです。
  このプロジェクトでは常に `move_to` で位置を指定して書きます。
- 元に戻す処理を **`Drop` に置く** のが重要です。`?` で早期 return しても、関数を抜けるときに必ず実行されます (RAII)。
- stdin が端末でない (パイプ等) と `tcgetattr` が失敗します。`main` はそのエラーを表示して終了コード 1 で終わります。

確認方法
- `cargo build -p p03_ansi_raw_starter` が通る。
- `cargo run -p p03_ansi_raw_starter < /dev/null` でエラーメッセージが出て終了する (端末が壊れない)。

## STEP 3: 代替スクリーン・カーソル非表示・パニックフック

やること
- `enable` の最後で `ENTER_ALT_SCREEN` と `HIDE_CURSOR` を出力して `flush` する。
- `restore` で `RESET`, `SHOW_CURSOR`, `LEAVE_ALT_SCREEN` を出力する (termios を戻す前に)。
- `install_panic_hook` を実装し、`main.rs` の `run` の先頭で呼ぶ。

コード例
```rust,noplayground
pub fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore();          // 先に端末を戻す
        default_hook(info); // それから通常のパニック表示
    }));
}
```

ポイント
- 代替スクリーンは vim や less が使う「別の画面」です。終了時に抜ければ、元のシェルの表示がそのまま戻ります。
- パニック時は **フック → unwind (Drop)** の順に実行されます。Drop だけだとパニックメッセージが代替スクリーンに書かれ、
  画面を戻した瞬間に消えてしまいます。だからフックの中で先に `restore()` します。
  `restore` は 2 回呼ばれても安全なように `Option::take` で作ってあります。
- 試しに `handle_key` の先頭に `if key == Key::Char('p') { panic!("test") }` を入れ、`p` を押して
  メッセージが普通の画面に表示されることを確かめましょう (確認したら消す)。

確認方法
- この時点ではまだキー解析が `todo!()` なので、何かキーを押すとパニックします。
  それでも **元の画面に戻り、パニックメッセージが読め、入力も普通にエコーされる** ことを確認してください。

## STEP 4: 端末サイズを ioctl で取得する

やること
- `window_size` を実装する。

コード例
```rust,noplayground
pub fn window_size() -> io::Result<(u16, u16)> {
    let mut ws = MaybeUninit::<libc::winsize>::zeroed();
    let r = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, ws.as_mut_ptr()) };
    if r != 0 {
        return Err(io::Error::last_os_error());
    }
    let ws = unsafe { ws.assume_init() };
    if ws.ws_col == 0 || ws.ws_row == 0 {
        return Err(io::Error::other("端末サイズが 0 です"));
    }
    Ok((ws.ws_col, ws.ws_row))
}
```

ポイント
- `ioctl` はデバイスに「命令番号 + データへのポインタ」を渡す汎用のシステムコールです。
  `TIOCGWINSZ` (= Terminal IO Control Get WINdow SiZe) を渡すとカーネルが `winsize` 構造体を埋めてくれます。
- `unsafe` ブロックには「なぜ安全か」を `// SAFETY:` コメントで書く習慣を付けましょう。

確認方法
- 実行すると最下行 (ステータス行) が端末の一番下に表示される (まだキーを押すとパニックします)。

## STEP 5: キー入力を解析する (ASCII・Ctrl・UTF-8)

やること
- `key.rs` の `parse_key` と `parse_utf8` を実装する (ESC の処理は次の STEP)。

コード例
```rust,noplayground
pub fn parse_key(input: &[u8]) -> Option<(Key, usize)> {
    let &first = input.first()?;
    let parsed = match first {
        ESC => parse_escape(input),
        b'\r' | b'\n' => (Key::Enter, 1),
        b'\t' => (Key::Tab, 1),
        0x7f | 0x08 => (Key::Backspace, 1),
        0x01..=0x1a => (Key::Ctrl((b'a' + first - 1) as char), 1),
        0x20..=0x7e => (Key::Char(first as char), 1),
        0x80..=0xff => parse_utf8(input),
        _ => (Key::Unknown(vec![first]), 1),
    };
    Some(parsed)
}

fn parse_utf8(input: &[u8]) -> (Key, usize) {
    let len = match input[0] {
        0xc0..=0xdf => 2,
        0xe0..=0xef => 3,
        0xf0..=0xf7 => 4,
        _ => return (Key::Unknown(vec![input[0]]), 1),
    };
    if input.len() < len {
        return (Key::Unknown(input.to_vec()), input.len());
    }
    match std::str::from_utf8(&input[..len]) {
        Ok(s) => (Key::Char(s.chars().next().unwrap()), len),
        Err(_) => (Key::Unknown(vec![input[0]]), 1),
    }
}
```

ポイント
- Ctrl + 英字は「その文字のコード & 0x1f」になります。Ctrl-A = 0x01, Ctrl-C = 0x03 ... Ctrl-Z = 0x1a。
  だから Tab (0x09) = Ctrl-I、Enter (0x0d) = Ctrl-M で、端末からは区別できません。
- raw モードでは ICRNL を切っているので Enter は `\r` (0x0d) で届きます。
- 戻り値に「消費したバイト数」を含めるのがポイント。1 回の `read` に複数のキーが入っていても、
  `parse_keys` のように先頭から順に切り出せます。
- UTF-8 は先頭バイトの上位ビットで長さが決まります (`110xxxxx` = 2, `1110xxxx` = 3, `11110xxx` = 4)。

確認方法
- `cargo test -p p03_ansi_raw_starter key::` で `ascii_and_control` と `utf8` が通る (`arrows...` はまだ失敗)。

## STEP 6: エスケープシーケンスを解析する (矢印キー・Esc・Alt)

やること
- `parse_escape` と `parse_csi` を実装する。

コード例
```rust,noplayground
fn parse_escape(input: &[u8]) -> (Key, usize) {
    match input.get(1) {
        None => (Key::Esc, 1),
        Some(b'[') => parse_csi(input),
        Some(b'O') => match input.get(2) {
            Some(b'A') => (Key::Up, 3),
            Some(b'B') => (Key::Down, 3),
            Some(b'C') => (Key::Right, 3),
            Some(b'D') => (Key::Left, 3),
            Some(b'H') => (Key::Home, 3),
            Some(b'F') => (Key::End, 3),
            _ => (Key::Alt('O'), 2),
        },
        Some(&b) if (0x20..=0x7e).contains(&b) => (Key::Alt(b as char), 2),
        Some(_) => (Key::Esc, 1),
    }
}

fn parse_csi(input: &[u8]) -> (Key, usize) {
    let Some(end) = input[2..].iter().position(|b| (0x40..=0x7e).contains(b)).map(|p| p + 2) else {
        return (Key::Unknown(input.to_vec()), input.len());
    };
    let params = &input[2..end];
    let len = end + 1;
    let key = match (params, input[end]) {
        (_, b'A') => Key::Up,
        (_, b'B') => Key::Down,
        (_, b'C') => Key::Right,
        (_, b'D') => Key::Left,
        (_, b'H') => Key::Home,
        (_, b'F') => Key::End,
        (b"1" | b"7", b'~') => Key::Home,
        (b"4" | b"8", b'~') => Key::End,
        (b"3", b'~') => Key::Delete,
        (b"5", b'~') => Key::PageUp,
        (b"6", b'~') => Key::PageDown,
        _ => Key::Unknown(input[..len].to_vec()),
    };
    (key, len)
}
```

ポイント
- 矢印キーは `ESC [ A` の 3 バイトで届きます。CSI シーケンスは「`ESC [` + パラメータ (数字と `;`) + 終端文字 1 バイト」という形です。
  Shift+↑ は `ESC [ 1 ; 2 A` のようにパラメータが付きますが、終端文字 `A` を見れば矢印と分かります。
- **Esc キー単独と、エスケープシーケンスの始まりは同じ `0x1b`** です。ここでは「1 回の read で ESC だけが届いたら Esc キー」
  とみなしています (本物のライブラリも、短いタイムアウトで次のバイトを待つという似た方法をとります)。
- タプルとバイト文字列リテラル (`b"3"`) へのパターンマッチで、表のような分岐が素直に書けます。

確認方法
- `cargo test -p p03_ansi_raw_starter key::` がすべて通る。

## STEP 7: アプリの更新とリサイズ

やること
- `app.rs` の `handle_key` を実装する (`render` は実装済み。ANSI で画面を組み立てる例として読んでおく)。
- `main.rs` のループにリサイズ検出を加える。

コード例
```rust,noplayground
pub fn handle_key(&mut self, key: Key) {
    match key {
        Key::Char('q') | Key::Ctrl('c') | Key::Ctrl('q') => self.quit = true,
        Key::Up | Key::Char('k') => self.y = self.y.saturating_sub(1),
        Key::Down | Key::Char('j') => self.y = self.y.saturating_add(1),
        Key::Left | Key::Char('h') => self.x = self.x.saturating_sub(2),
        Key::Right | Key::Char('l') => self.x = self.x.saturating_add(2),
        Key::Home => self.x = 0,
        Key::End => self.x = self.max_x(),
        Key::PageUp => self.y = self.min_y(),
        Key::PageDown => self.y = self.max_y(),
        _ => {}
    }
    self.last_key = Some(key);
    self.clamp();
}
```

```rust,noplayground
if let Ok((c, r)) = terminal::window_size()
    && (c, r) != (app.cols, app.rows)
{
    app.resize(c, r);
    dirty = true;
}
```

ポイント
- ISIG を切ったので Ctrl-C はシグナルではなく `0x03` というただのバイトとして届きます。自分で終了処理に割り当てましょう。
- ループは「描画 (必要なときだけ) → 入力待ち (最大 0.1 秒) → 更新」の繰り返しです。VTIME のおかげで入力が無くても
  0.1 秒ごとにループが回るので、そのたびにサイズを問い合わせてリサイズを検出できます (ポーリング方式)。
- 状態 (`App`) と描画 (`render` が返す `String`) は端末に触れないので、`cargo test` で検証できます。
  端末を触るのは `main.rs` と `terminal.rs` だけ、という分離を次のプロジェクト以降も続けます。

確認方法
- `cargo test -p p03_ansi_raw_starter` がすべて通る。
- `cargo run -p p03_ansi_raw_starter` で箱が動き、`q` で終了して元の画面が戻る。日本語や Delete キーを押してバイト列を観察しよう。
- 端末ウィンドウの大きさを変えると、ステータス行の `size:` が変わり箱が画面内に収まる。

## 発展課題

- `SIGWINCH` シグナル (端末サイズ変更) を `libc::signal` で受け取り、ポーリングをやめる。
- 毎回 `CLEAR_SCREEN` で全消去するとちらつくことがある。前回の箱の位置だけを空白で消してから描くように変える。
- マウスイベントを有効にする (`\x1b[?1000h\x1b[?1006h`) と、クリックが `ESC [ < b ; x ; y M` で届く。パーサに追加しよう。
- 箱の中に入力した文字を書けるようにする (日本語は表示幅 2 なので、第 16 章の表示幅の話を思い出そう)。
