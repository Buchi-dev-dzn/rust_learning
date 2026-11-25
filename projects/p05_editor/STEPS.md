# kilo 風テキストエディタを作る

C 言語で約 1000 行の小さなエディタ [kilo](https://github.com/antirez/kilo) を、Rust + crossterm で作り直します。
ファイルを開いて、カーソルで移動し、日本語を含む文字を入力・削除して、保存・検索できるところまで作ります。

## 完成イメージ

```text
//! kilo 風テキストエディタのコアロジック。
//!
pub mod editor;
pub mod row;
~
~
src/lib.rs - 13 行 (変更あり)                         3/13   ← ステータスバー (反転表示)
ヘルプ: Ctrl-S = 保存 | Ctrl-Q = 終了 | Ctrl-F = 検索       ← メッセージバー
```

| キー | 動作 |
| --- | --- |
| 矢印 / Home / End / PageUp / PageDown | カーソル移動 |
| 文字 / Enter / Backspace / Delete | 編集 |
| Ctrl-S | 保存 (ファイル名がなければ入力を求める) |
| Ctrl-F | インクリメンタル検索 (矢印で次/前、Enter で確定、ESC で中止) |
| Ctrl-Q | 終了 (未保存なら 2 回押す) |

## 学ぶこと

- raw モードと「パニックしても端末を戻す」後片付け
- **文字インデックス・バイトインデックス・画面の列** の 3 つの座標を区別すること (UTF-8 と全角文字)
- 縦横のスクロール
- 端末 I/O (`main.rs`) と純粋なロジック (`lib.rs`) を分けて、ロジックを単体テストする設計

## ファイル構成

```text
projects/p05_editor/starter/src/
├── main.rs    # 端末: raw モード、キー変換、描画 (crossterm を使うのはここだけ)
├── lib.rs
├── width.rs   # 文字の表示幅
├── row.rs     # 1 行分のテキスト
└── editor.rs  # エディタの状態とロジック
```

starter の `TODO(STEP n)` を順番に埋めていきます。各 STEP の最後にテストや実行で確認しましょう。
テストは `cargo test -p p05_editor_starter <テスト名の一部>` で絞り込めます。
完成版は `projects/p05_editor/` にあります (詰まったら覗いて構いません)。

## STEP 1: raw モードと終了キー

### やること

`main.rs` の `TerminalGuard`・`restore_terminal`・`to_key`・`draw`・`run` を書き、
「`~` が並ぶ画面が出て、Ctrl-Q で終了する」状態にします。`editor.rs` の `process_key` には Ctrl-Q だけ足します。

### ポイント

raw モードに入ったまま終了すると、ターミナルが壊れたまま (入力が見えない等) になります。
**Drop** と **パニックフック** の 2 つで必ず元に戻します。

```rust,noplayground
struct TerminalGuard;

impl TerminalGuard {
    fn new() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal(); // Show + LeaveAlternateScreen + disable_raw_mode
    }
}

// main の最初で
let default_hook = std::panic::take_hook();
std::panic::set_hook(Box::new(move |info| {
    restore_terminal();   // 先に端末を戻さないと、パニックメッセージが崩れて読めない
    default_hook(info);
}));
```

crossterm のキーイベントは、エディタ専用の `Key` 型に変換してからロジックに渡します。
こうすると `editor.rs` は crossterm を知らなくてよく、テストで `Key::Char('a')` を直接渡せます。

```rust,noplayground
fn to_key(ev: KeyEvent) -> Option<Key> {
    if ev.kind != KeyEventKind::Press {
        return None; // Windows では離したときのイベントも来る
    }
    let key = match ev.code {
        KeyCode::Char(c) if ev.modifiers.contains(KeyModifiers::CONTROL) => Key::Ctrl(c.to_ascii_lowercase()),
        KeyCode::Char(c) => Key::Char(c),
        KeyCode::Tab => Key::Char('\t'),
        KeyCode::Enter => Key::Enter,
        // ... 矢印・Home/End・PageUp/PageDown・Backspace・Delete・Esc
        _ => return None,
    };
    Some(key)
}
```

描画は毎回「カーソルを隠す → 各行を書く → カーソルを置く → 表示」の順にすると、ちらつきません。
`queue!` でためて、最後に 1 回 `flush()` します。

```rust,noplayground
queue!(out, cursor::Hide)?;
for (y, line) in screen.rows.iter().enumerate() {
    queue!(out, cursor::MoveTo(0, y as u16), Print(line), terminal::Clear(ClearType::UntilNewLine))?;
}
queue!(out, cursor::MoveTo(screen.cursor.0 as u16, screen.cursor.1 as u16), cursor::Show)?;
out.flush()
```

`run` は `while !editor.should_quit { render → draw → event::read → process_key }` です。
`Event::Resize(w, h)` が来たら `editor.resize` を呼びます。

> 豆知識: Ctrl-Q は、通常のターミナル (cooked モード) では「出力再開 (XON)」として端末ドライバに食べられてしまいます。
> raw モードは IXON も無効にするので、アプリに届くようになります。

### 確認方法

```sh
cargo run -p p05_editor_starter
```

`~` が縦に並び、Ctrl-Q で元のシェルに戻れば OK。終了後にシェルの表示が崩れていないことも確認します。

## STEP 2: 1 行のテキストと表示幅 (UTF-8 と全角文字)

### やること

`width.rs` と `row.rs` の `width_at`・`byte_index`・`cx_to_rx`・`rx_to_cx`・`render` を実装します。

### ポイント: 3 つの座標

`"aあb"` という行を考えます。

| 文字 | a | あ | b |
| --- | --- | --- | --- |
| 文字インデックス `cx` | 0 | 1 | 2 |
| バイト位置 | 0 | 1 | 4 |
| 画面の列 `rx` | 0 | 1〜2 | 3 |

- 編集 (挿入・削除) は **cx** で考える。`String` を操作するときは **バイト位置** に変換する (`String::insert` にバイト位置以外を渡すとパニック)。
- 画面に描くとき・カーソルを置くときは **rx** を使う。全角文字は 2 列、タブは次のタブストップ (4 の倍数) まで。

```rust,noplayground
pub fn byte_index(&self, cx: usize) -> usize {
    self.chars.char_indices().nth(cx).map(|(i, _)| i).unwrap_or(self.chars.len())
}

pub fn cx_to_rx(&self, cx: usize) -> usize {
    let mut rx = 0;
    for c in self.chars.chars().take(cx) {
        rx += width_at(c, rx); // タブの幅は「今いる列」で変わるので rx を渡す
    }
    rx
}
```

表示幅の判定は、本来は Unicode の East Asian Width 表を使いますが、ここでは主要な範囲を `matches!` で書きます。

```rust,noplayground
fn is_wide(cp: u32) -> bool {
    matches!(cp, 0x1100..=0x115F | 0x2E80..=0x303E | 0x3041..=0x33FF | 0x4E00..=0x9FFF
        | 0xAC00..=0xD7A3 | 0xFF00..=0xFF60 | 0x1F300..=0x1F64F /* ... */)
}
```

`render(coloff, width)` は「画面の `coloff` 列目から `width` 列ぶん」を切り出します。
全角文字が左端・右端で半分だけはみ出すときは、**見えている列数ぶんの空白** にします。
半分だけ文字を出すことはできないので、こうしないと 1 列ずれて画面が崩れます。

```rust,noplayground
let start = rx;          // この文字の開始列
rx += w;                 // 終了列 (この列は含まない)
if rx <= coloff { continue; }      // 完全に左
if start >= end { break; }         // 右端を超えた
let fully_visible = start >= coloff && rx <= end;
if c == '\t' || !fully_visible {
    let visible = rx.min(end) - start.max(coloff);
    out.extend(std::iter::repeat_n(' ', visible));
}
```

### 確認方法

```sh
cargo test -p p05_editor_starter width::
cargo test -p p05_editor_starter row::tests::cx_rx
cargo test -p p05_editor_starter row::tests::render
cargo test -p p05_editor_starter row::tests::tabs
```

## STEP 3: ファイルを開いて描画する

### やること

- `load_text` / `open`: ファイルを行に分割して `rows` に入れる。**ファイルが存在しなければ空の新規バッファ** にする。
- `render`: 各行を `row.render(coloff, screen_cols)` で描く。バッファが空なら画面の 1/3 の高さにようこそメッセージ。
- ステータスバー (ファイル名・行数・変更あり・現在行) とメッセージバー (5 秒で消える) を作る。
- `main.rs`: 引数のファイルを開き、`draw` でステータスバーを反転表示 (`SetAttribute(Attribute::Reverse)`) する。

### ポイント

```rust,noplayground
pub fn open(&mut self, path: impl Into<PathBuf>) -> io::Result<()> {
    let path = path.into();
    match fs::read_to_string(&path) {
        Ok(text) => self.load_text(&text),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            self.load_text("");
            self.set_message(format!("新規ファイル: {}", path.display()));
        }
        Err(e) => return Err(e), // 権限がない等は本当のエラー
    }
    self.filename = Some(path);
    Ok(())
}
```

ステータスバーは「左寄せの情報 + 空白 + 右寄せの行番号」を **ちょうど画面幅** にします。
幅の計算には `len()` (バイト数) ではなく `str_width` を使います。日本語のファイル名で右端がずれないように。

```rust,noplayground
let mut line = truncate_to_width(&left, cols);
let used = str_width(&line);
let rw = str_width(&right);
if used + rw < cols {
    line.extend(std::iter::repeat_n(' ', cols - used - rw));
    line.push_str(&right);
} else {
    line.extend(std::iter::repeat_n(' ', cols - used));
}
```

### 確認方法

```sh
cargo test -p p05_editor_starter welcome
cargo test -p p05_editor_starter status_bar
cargo run -p p05_editor_starter -- projects/p05_editor/STEPS.md
cargo run -p p05_editor_starter -- no_such_file.txt   # 新規ファイル扱いになる
```

## STEP 4: カーソル移動とスクロール

### やること

`rx`・`move_cursor`・`scroll`・`Row::width_of` を実装し、`process_key` から矢印キーなどを `move_cursor` に渡します。
`render` の `cursor` を `(rx - coloff, cy - rowoff)` にします。

### ポイント

- `cy` は `rows.len()` (最終行の **次**) まで動けるようにします。ここで文字を打つと新しい行が追加されます。
- 左右キーは行をまたぎます (行頭で ← なら前の行の行末へ)。
- 上下キーは **画面上の列 rx を保って** 移動します。cx を保つと、全角文字の行で縦にずれてしまいます。

```rust,noplayground
Key::Up | Key::Down => {
    let rx = self.rx();
    if key == Key::Up && self.cy > 0 { self.cy -= 1; }
    else if key == Key::Down && self.cy < self.rows.len() { self.cy += 1; }
    self.cx = self.rows.get(self.cy).map_or(0, |r| r.rx_to_cx(rx));
}
```

スクロールは「カーソルが画面外に出たら、オフセットを動かして画面内に戻す」だけです。
横方向は、カーソル位置の文字が全角なら 2 列ぶん収まるようにします。

```rust,noplayground
pub fn scroll(&mut self) {
    let rx = self.rx();
    if self.cy < self.rowoff { self.rowoff = self.cy; }
    if self.cy >= self.rowoff + self.screen_rows { self.rowoff = self.cy + 1 - self.screen_rows; }
    let w = self.rows.get(self.cy).map_or(1, |r| r.width_of(self.cx));
    if rx < self.coloff { self.coloff = rx; }
    if rx + w > self.coloff + self.screen_cols { self.coloff = rx + w - self.screen_cols; }
}
```

### 確認方法

```sh
cargo test -p p05_editor_starter cursor_wraps
cargo test -p p05_editor_starter vertical
cargo test -p p05_editor_starter horizontal_scroll
cargo run -p p05_editor_starter -- projects/p05_editor/STEPS.md
```

長いファイルで PageDown、日本語の行で上下移動・行末 (End) を試します。カーソルが文字の境目に正しく乗れば OK。

## STEP 5: 文字の挿入・削除・改行

### やること

`Row::insert_char`・`delete_char`・`split_off` と、`Editor::insert_char`・`insert_newline`・`delete_back`・`delete_forward` を実装し、
`process_key` に Enter / Backspace / Delete / 文字入力をつなぎます。変更したら `dirty = true` にします。

### ポイント

```rust,noplayground
pub fn insert_newline(&mut self) {
    if self.cy == self.rows.len() {
        self.rows.push(Row::default());
    } else {
        let rest = self.rows[self.cy].split_off(self.cx); // カーソル以降を切り取る
        self.rows.insert(self.cy + 1, Row::new(rest));
    }
    self.cy += 1;
    self.cx = 0;
    self.dirty = true;
}

pub fn delete_back(&mut self) {
    // ...
    if self.cx > 0 {
        self.rows[self.cy].delete_char(self.cx - 1);
        self.cx -= 1;
    } else if self.cy > 0 {
        let row = self.rows.remove(self.cy);       // 今の行を取り出して
        self.cy -= 1;
        self.cx = self.rows[self.cy].len();        // 前の行の行末に移動し
        self.rows[self.cy].append(&row.chars);     // 連結する
    }
}
```

文字入力は `Key::Char(c) if c == '\t' || !c.is_control()` だけ受け付けます。
日本語入力 (IME) の確定文字列も 1 文字ずつ `Key::Char` で届きます。

### 確認方法

```sh
cargo test -p p05_editor_starter insert
cargo test -p p05_editor_starter newline
cargo test -p p05_editor_starter backspace
cargo test -p p05_editor_starter edit_japanese
```

実行して、日本語を打って Backspace で 1 文字ずつ (バイト単位ではなく) 消えることを確認します。

## STEP 6: 保存と終了確認

### やること

- `text` / `save`: 各行に `\n` を付けてファイルに書く。書いたバイト数をメッセージに出す。
- Ctrl-S: ファイル名がなければ「保存先:」プロンプトでファイル名を入力させる (`Prompt` / `PromptKind::SaveAs`)。
- Ctrl-Q: `dirty` なら警告を出し、**続けてもう一度** Ctrl-Q を押したら終了 (`quit_confirm`)。

### ポイント

プロンプト中はキーの意味が変わります (文字は入力欄へ、Enter で確定、ESC で中止)。
`process_key` の先頭で分岐させます。

```rust,noplayground
pub fn process_key(&mut self, key: Key) {
    if self.prompt.is_some() {
        self.process_prompt_key(key);
        return;
    }
    if key != Key::Ctrl('q') {
        self.quit_confirm = false; // 別のキーを押したら確認はやり直し
    }
    match key {
        Key::Ctrl('q') => {
            if self.dirty && !self.quit_confirm {
                self.set_message("警告: 未保存の変更があります。もう一度 Ctrl-Q で破棄して終了します");
                self.quit_confirm = true;
            } else {
                self.should_quit = true;
            }
        }
        // ...
    }
}
```

`process_prompt_key` では `self.prompt.take()` で一度取り出してから処理し、続けるなら `self.prompt = Some(p)` で戻します。
こうすると「自分自身を借用しながら自分のフィールドを変更する」借用エラーを避けられます。
プロンプト中のカーソルはメッセージバー (画面の最終行) に置きます。

### 確認方法

```sh
cargo test -p p05_editor_starter save
cargo test -p p05_editor_starter quit
cargo run -p p05_editor_starter -- /tmp/memo.txt
```

文字を打って Ctrl-Q → 警告、もう一度 Ctrl-Q → 終了。Ctrl-S 後は 1 回で終了でき、`cat /tmp/memo.txt` で内容を確認できます。

## STEP 7: インクリメンタル検索

### やること

Ctrl-F で「検索:」プロンプトを開き、**1 文字打つごとに** 一致する行へカーソルを移動します (`find_step`)。
→ / ↓ で次、← / ↑ で前、Enter で確定、ESC で検索前の位置に戻ります。

### ポイント

- `Prompt.saved` に検索開始時の `(cx, cy, rowoff, coloff)` を保存しておき、ESC で戻す。
- `last_match` (前回一致した行) から 1 行ずつ進み、末尾まで来たら先頭へ折り返す。
- `str::find` はバイト位置を返すので、`row.chars[..byte].chars().count()` で cx に変換する。

```rust,noplayground
let n = self.rows.len();
let mut current = p.last_match;
for _ in 0..n {
    let next = match (current, p.forward) {
        (None, true) => 0,
        (None, false) => n - 1,
        (Some(i), true) => (i + 1) % n,
        (Some(i), false) => (i + n - 1) % n,
    };
    current = Some(next);
    if let Some(byte) = self.rows[next].chars.find(&p.input) {
        p.last_match = Some(next);
        self.cy = next;
        self.cx = self.rows[next].chars[..byte].chars().count();
        self.rowoff = n; // 次の scroll() で一致行が画面の一番上に来る (kilo の小技)
        return;
    }
}
```

### 確認方法

```sh
cargo test -p p05_editor_starter search
cargo test -p p05_editor_starter      # 全テストが通ること
cargo run -p p05_editor_starter -- projects/p05_editor/STEPS.md
```

Ctrl-F で `STEP` と打ち、↓ を押すたびに次の STEP 見出しへジャンプすれば完成です。

## 発展課題

- **検索結果のハイライト**: 一致部分を色付きで表示する (`Screen` の行を「文字列 + 色の範囲」に拡張する)。
- **同じ行の 2 つ目以降の一致** にも ↓ で移動できるようにする。
- **シンタックスハイライト**: 数字・文字列・キーワード (`fn`, `let` ...) を色分けする (kilo の後半の章)。
- **Undo / Redo**: 編集操作を `enum Edit` として記録し、逆操作を適用する。
- **行番号の表示** と、画面幅に合わせた **折り返し表示**。
- 表示幅の判定を Unicode の East Asian Width 表に基づいて正確にする (絵文字の ZWJ 結合など)。
