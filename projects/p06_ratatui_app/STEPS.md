# ratatui でファイルエクスプローラを作る

ratatui を使って、左にディレクトリの一覧、右にファイルのプレビューを表示する 2 ペインのファイルブラウザを作ります。
crossterm だけで書いた p04/p05 と比べて、「画面をどう組み立てるか」だけに集中できることを体験しましょう。

## 完成イメージ

```text
 ファイルエクスプローラ /Users/you/rust_learning                      ← ヘッダ
┌ 12 件 ─────────────────────┐┌ Cargo.toml ──────────────────────────┐
│> course/                   ││[workspace]                           │
│  docs/                     ││resolver = "3"                        │
│  projects/                 ││members = [                           │
│  Cargo.toml                ││    "studio",                         │
│  README.md                 ││                                      │
└────────────────────────────┘└───────────────────────────── 1/30 ───┘
 j/k:移動  Enter:開く  h:上へ  J/K:プレビュー  ?:ヘルプ  q:終了          ← フッタ
```

`?` を押すと中央にヘルプのポップアップが重なって表示されます。

## 学ぶこと

- `ratatui::init()` / `ratatui::restore()` と、即時モード (毎フレーム全部描き直す) の描画ループ
- `Layout` と `Constraint` (Length / Percentage / Fill) による画面分割
- `List` + `ListState` (StatefulWidget) による選択とスクロール
- `Paragraph` のスクロール、`Block` の枠とタイトル、`Clear` を使ったポップアップ
- 状態 (`app.rs`) と描画 (`ui.rs`) の分離、`TestBackend` を使った描画のテスト

## ファイル構成

```text
projects/p06_ratatui_app/starter/src/
├── main.rs   # 端末の初期化とイベントループ
├── lib.rs
├── app.rs    # 状態とキー操作 (ratatui の描画は知らない)
└── ui.rs     # 状態を読んで画面を組み立てる (状態は変えない)
```

テストは `cargo test -p p06_ratatui_app_starter <名前の一部>` で絞り込めます。完成版は `projects/p06_ratatui_app/` にあります。

## STEP 1: ratatui の初期化とレイアウト

### やること

- `main.rs`: `ratatui::init()` で端末を準備し、`run` でイベントループを回し、最後に `ratatui::restore()`。
- `ui.rs`: `draw` で画面をヘッダ (1 行)・本体・フッタ (1 行) に分け、`draw_header` と `draw_footer` を書く。

### ポイント

`ratatui::init()` は raw モード + 代替スクリーンに入り、**パニック時に端末を戻すフックも設定** してくれます。
p04/p05 で手書きした `TerminalGuard` とパニックフックがこの 1 行に入っています。

```rust,noplayground
fn main() -> io::Result<()> {
    let mut app = App::new(".")?;          // 端末を触る前に失敗しうる処理を済ませる
    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut app);
    ratatui::restore();                    // ? で早期 return する前に必ず戻す
    result
}

fn run(terminal: &mut DefaultTerminal, app: &mut App) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;
        if let Event::Key(key) = event::read()? {
            app.handle_key(key);
        }
    }
    Ok(())
}
```

ratatui は **即時モード** です。毎フレーム画面全体を描き直すコードを書きますが、
実際に端末へ送られるのは前フレームとの差分だけです (この仕組みは p07 で自作します)。

`Layout` は配列の分割結果を `areas()` で受け取ると便利です。

```rust,noplayground
let [header, body, footer] = Layout::vertical([
    Constraint::Length(1), // ちょうど 1 行
    Constraint::Fill(1),   // 残り全部
    Constraint::Length(1),
])
.areas(frame.area());

frame.render_widget(
    Paragraph::new(" ファイルエクスプローラ ").style(Style::new().fg(Color::Black).bg(Color::Cyan)),
    header,
);
```

### 確認方法

```sh
cargo run -p p06_ratatui_app_starter
```

上下に色付きの行が出て、`q` で終了できれば OK (中央はまだ空)。

## STEP 2: ディレクトリを読んで List に表示する

### やること

- `app.rs`: `read_entries` と `change_dir` を実装する。
- `ui.rs`: 本体を左右 (40% / 60%) に分け、`draw_list` で `List` を描く。

### ポイント

並び順は「ディレクトリが先、その中は名前順」。`bool` の比較は `false < true` なので、`b.is_dir.cmp(&a.is_dir)` と逆にします。

```rust,noplayground
entries.sort_by(|a, b| {
    b.is_dir
        .cmp(&a.is_dir)
        .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
});
```

`change_dir` は **読み込みに成功してから** 状態を書き換えます。権限のないディレクトリに入ろうとして失敗しても、画面が壊れないようにするためです。

```rust,noplayground
let entries = read_entries(&dir)?; // 失敗したらここで Err を返し、何も変えない
self.cwd = dir;
self.entries = entries;
self.list_state.select(if self.entries.is_empty() { None } else { Some(0) });
```

`List` は `render_stateful_widget` で描きます。`ListState` (どれが選択されているか・どこまでスクロールしたか) は
ウィジェットではなく **App が持ち続ける** 状態です。

```rust,noplayground
let items: Vec<ListItem> = app.entries.iter().map(|e| {
    if e.is_dir {
        ListItem::new(format!("{}/", e.name)).style(Style::new().fg(Color::Blue).bold())
    } else {
        ListItem::new(e.name.as_str())
    }
}).collect();
let list = List::new(items).block(Block::bordered().title(format!(" {} 件 ", app.entries.len())));
frame.render_stateful_widget(list, area, &mut app.list_state);
```

### 確認方法

```sh
cargo test -p p06_ratatui_app_starter entries_are_sorted
cargo run -p p06_ratatui_app_starter
```

左のペインにリポジトリ直下のディレクトリとファイルが並べば OK。

## STEP 3: 選択を動かす (ListState)

### やること

`select` / `next` / `previous` を実装し、`handle_key` で j/k/↓/↑、g/G (先頭/末尾) をつなぎます。
`draw_list` に `highlight_style` と `highlight_symbol("> ")` を追加します。

### ポイント

末尾で `j` を押したら先頭に戻る (ラップする) ようにします。`% len` が便利です。

```rust,noplayground
pub fn next(&mut self) {
    let len = self.entries.len();
    if len == 0 { return; }
    let i = self.list_state.selected().map_or(0, |i| (i + 1) % len);
    self.select(i);
}
```

一覧が画面より長いときのスクロールは、`ListState` が選択位置を見て自動でやってくれます。
(ratatui 0.30 には `select_next()` もありますが、末尾でラップしないので今回は自分で書きます。)

### 確認方法

```sh
cargo test -p p06_ratatui_app_starter move_selection
cargo run -p p06_ratatui_app_starter -- projects
```

## STEP 4: プレビューとスクロール

### やること

- `load_preview`: ディレクトリなら子の一覧、テキストなら内容、NUL を含めば「バイナリファイル」と表示する文字列を作る。
- `update_preview`: 選択が変わるたびに呼び、スクロール位置を 0 に戻す。`change_dir` と `select` の最後で呼ぶ。
- `scroll_preview`: J/K で 1 行、PageDown/PageUp で 10 行。範囲外に行かないよう `clamp`。
- `draw_preview`: `Paragraph::scroll` で表示位置をずらす。

### ポイント

- 巨大なファイルで固まらないよう、`Read::take(PREVIEW_LIMIT)` で先頭だけ読みます。
- 途中で切ると UTF-8 の文字が壊れることがあるので `String::from_utf8_lossy` で変換します。
- タブ文字は表示幅が決まらず表示が乱れるので、空白 4 つに置き換えます。

```rust,noplayground
let preview = Paragraph::new(app.preview.as_str())
    .block(
        Block::bordered()
            .title(format!(" {} ", name))
            .title_bottom(Line::from(format!(" {}/{} ", app.preview_scroll + 1, lines)).right_aligned()),
    )
    .scroll((app.preview_scroll, 0)); // (縦, 横)
frame.render_widget(preview, area);
```

`draw` は `&mut App` を受け取っていますが、ペインごとの関数は必要なものだけ借ります
(`draw_list` は ListState を更新するので `&mut`、`draw_preview` は `&App`)。

### 確認方法

```sh
cargo test -p p06_ratatui_app_starter preview
cargo run -p p06_ratatui_app_starter
```

`Cargo.toml` を選ぶと右に内容が出て、`J` で 1 行ずつ送れれば OK。

## STEP 5: ディレクトリの移動

### やること

`enter` (Enter / l / →) と `go_up` (Backspace / h / ←) を実装します。

### ポイント

親に戻ったとき、**さっきまでいたディレクトリが選択されている** と快適です。
`change_dir` の `select_name` に元のディレクトリ名を渡します。

```rust,noplayground
pub fn go_up(&mut self) {
    let Some(parent) = self.cwd.parent().map(Path::to_path_buf) else {
        self.status = "これ以上は上がれません".to_string();
        return;
    };
    let from = self.cwd.file_name().map(|n| n.to_string_lossy().into_owned());
    if let Err(e) = self.change_dir(parent, from.as_deref()) {
        self.status = format!("開けません: {e}");
    }
}
```

`enter` では `self.selected()` が `&Entry` (self からの借用) を返すので、そのまま `self.change_dir(...)` を呼ぶと
借用エラーになります。`.cloned()` で複製してから使いましょう。

### 確認方法

```sh
cargo test -p p06_ratatui_app_starter enter_and_go_up
cargo run -p p06_ratatui_app_starter
```

`projects/` に入って `h` で戻ると、`projects/` が選択された状態に戻れば OK。

## STEP 6: ヘルプのポップアップと描画テスト

### やること

- `?` で `show_help` を切り替え、表示中は `?` / Esc で閉じる・`q` で終了以外のキーを無視する。
- `draw_help`: 画面中央に枠付きのヘルプを重ねる。
- (おまけ) `TestBackend` で描画結果をテストする。

### ポイント

ポップアップは「最後に描いたものが上に来る」だけです。ただし先に `Clear` で下の文字を消さないと、
下の一覧が透けて見えます。

```rust,noplayground
let area = frame.area().centered(Constraint::Percentage(60), Constraint::Length(11));
frame.render_widget(Clear, area);
frame.render_widget(
    Paragraph::new(lines).block(Block::bordered().title(" ヘルプ (Esc で閉じる) ")),
    area,
);
```

`TestBackend` を使うと、端末なしで「画面に何が描かれたか」をテストできます。

```rust,noplayground
let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
terminal.draw(|f| draw(f, &mut app)).unwrap();
let buf = terminal.backend().buffer();
let first_line: String = (0..80).map(|x| buf[(x, 0)].symbol()).collect();
assert!(first_line.contains("ファイル"));
```

全角文字は 2 セルを使い、右側のセルは空白になる点に注意 (完成版の `ui.rs` のテストを参照)。

### 確認方法

```sh
cargo test -p p06_ratatui_app_starter        # 全テストが通ること
cargo run -p p06_ratatui_app_starter
```

`?` でヘルプが中央に出て、Esc で閉じれば完成です。

## 発展課題

- 隠しファイル (`.` で始まる名前) の表示/非表示を `.` キーで切り替える。
- ファイルサイズや更新日時を一覧の右側に表示する (`Table` ウィジェットを使う)。
- `/` で名前の絞り込み (入力中の文字列をフッタに表示)。
- プレビューに `Scrollbar` ウィジェットを付ける。
- `.rs` ファイルのキーワードを色付けする (`Line` / `Span` を組み立てる)。
- マウスのクリック・スクロールに対応する (`EnableMouseCapture` と `Event::Mouse`)。
