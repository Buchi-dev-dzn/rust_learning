# 自作 TUI フレームワーク

## 概要

`libc` と標準ライブラリだけで端末を制御し、セルバッファ・ANSI 差分描画・レイアウト・ウィジェットを作ります。最後に、その上でタブ付き ToDo アプリを動かします。対象: 第22章「自作 TUI フレームワーク」。`ratatui` や `crossterm` は使いません。

完成版: `cargo run -p p07_own_tui`。練習用: `projects/p07_own_tui/starter/` (`cargo build -p p07_own_tui_starter`)。starter はビルドできますが、`todo!()` が残っている間は実行・テスト時にパニックします。完成版の `src/` は詰まったときの模範解答です。

## 完成イメージ

枠付きタブ「タスク / ヘルプ / 情報」、選択行が強調されるタスクリスト、達成率のゲージ、入力用ポップアップ、ステータス行のスピナーを表示します。`j/k` または矢印で選択、Space で完了切替、`a` で日本語のタスクを追加、`d` で削除、Tab でタブ移動、`q` で終了。終了・エラー・パニックの際に端末を元に戻します。

## 学ぶこと

- 文字列のバイト数・文字数と端末セル数の違い、および全角文字の右半分の扱い
- `Rect` と `Layout` で位置を計算し、`Widget` トレイトで描画を共通化する方法
- 画面をメモリ上で組み立て、前フレームとの差分を ANSI に変換する方法
- `termios` / `poll` / `ioctl` による raw 入力、サイズ取得と後片付け
- 端末 I/O とテスト可能な純粋ロジックの分離

## STEP 1: 層とテストの入口を確認する

### やること

`starter/src/lib.rs` の公開モジュールを確認します。`width → geometry / style → buffer → render / layout / widgets → terminal → demo` の順に依存します。`main.rs` は `Terminal::draw` に描画関数を渡し、`Events::next` の結果で状態を更新します。端末に直接書き込むのは `terminal.rs` だけです。

### コード例/ポイント

```rust
// 描画関数は端末なしでも呼べる
// let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
// app.render(&mut buf);
```

スターターのテストは最初は不合格です。`todo!()` は完成前でもコンパイルできるので、まず `cargo build -p p07_own_tui_starter` を試してください。完成版のテストでは幾何・色・描画・入力・デモの各層を別々に検査しています。

### 確認方法

`cargo build -p p07_own_tui_starter` が成功すること。`cargo test -p p07_own_tui_starter` は今は失敗して構いません。

## STEP 2: 表示幅と Rect を理解する

### やること

`width.rs` の `char_width` を読んで、`'a'` は 1 セル、`'あ'` は 2 セルであることを確認します。`geometry.rs` の `Rect::inner` と `intersection` の TODO を実装します。右端と下端は「含まない」境界です。狭い領域でも引き算がアンダーフローしないよう `saturating_sub` を使います。

### コード例/ポイント

```rust
// (2, 3) から幅 10・高さ 5 の領域
// inner(1) => (3, 4) から幅 8・高さ 3
// intersection は左上 = max、右下 = min
```

`width.rs` の幅判定は学習用の近似です。実運用では結合絵文字や端末ごとに異なる幅の文字には別途対応が必要です。

### 確認方法

`cargo test -p p07_own_tui_starter geometry::tests`。端に重なる矩形と、余白が大きすぎて空になる矩形を考えてみましょう。

## STEP 3: Style とセルバッファを作る

### やること

`style.rs` の `Color::sgr` / `Style::to_sgr` を実装します。ANSI の色は `ESC[0;31m` (赤)、背景は 10 を足して `44` (青)、256 色は `38;5;n`、RGB は `38;2;r;g;b`。毎回 `0` から始めて直前の書式を消します。

次に `buffer.rs` の `set_stringn` と `diff` を完成させます。`set_stringn` は表示幅を数え、領域外なら何も書かず、全角文字が右端で半分になる場合は書きません。`put` は全角文字の右半分を `skip` セルとして記録し、上書きされた相方を消します。`diff` は異なるセルだけを返し、`skip` を描画対象から外します。

### コード例/ポイント

```rust
// "aあb" の占有セルは 1 + 2 + 1 = 4
// Cell { ch: 'あ', skip: false } の右隣は skip: true
// 前後の Buffer を比較して Vec<(x, y, &Cell)> を返す
```

初回フレームでは旧セルが全部空白なので、空白から変わったセルだけを送れます。画面サイズが変わったときは `terminal.rs` が全消去して旧バッファを作り直します。

### 確認方法

`cargo test -p p07_own_tui_starter style::tests` と `cargo test -p p07_own_tui_starter buffer::tests`。全角を狭い端へ書くテストと、上書き後の差分テストが重要です。

## STEP 4: 差分を ANSI に変換する

### やること

`render.rs` の `diff_to_ansi` を実装します。各更新で必要なときだけカーソル移動 `ESC[行;列H` とスタイル変更を出力します。ANSI の座標は **1 始まり・行が先**、内部座標は **0 始まり・列が先**です。書いた文字の `char_width` だけ仮想カーソルを進め、隣接セルへの移動を省きます。出力がある場合にだけ最後に `ESC[0m` を足します。

### コード例/ポイント

```rust
// (x=2, y=1) に "abc" を連続出力するなら
// "\x1b[2;3H\x1b[0mabc\x1b[0m"
```

端末へ書く前に `String` を組み立てるので、実際の端末なしでバイト列を比較できます。

### 確認方法

`cargo test -p p07_own_tui_starter render::tests`。隣接セルで移動が 1 回、スタイルが同じなら SGR も 1 回だけであることを確認。

## STEP 5: Widget トレイトと部品を使う

### やること

`widgets.rs` の `wrap_line` と `List::offset` の TODO を実装します。`Widget::render(&self, Rect, &mut Buffer)` は端末を知らない共通インターフェース。`Block` は枠とタイトル、`Paragraph` は折り返し、`List` は選択行、`Tabs` は見出し、`Gauge` は達成率、`Clear` はポップアップの下を消します。まず `Block::render` と `List::render` を読んでください。

### コード例/ポイント

```rust
// 選択位置 sel が高さ h の外なら、末尾が見える最小の offset は sel + 1 - h
// ただし h == 0 や選択なしでは 0
// 日本語は chars().count() ではなく char_width で折り返す
```

ウィジェットには画面状態を保持せず、アプリ側の `selected` などを渡して毎フレーム作り直します。

### 確認方法

`cargo test -p p07_own_tui_starter widgets::tests`。狭い領域、全角文字、選択項目が画面外にある場合を見てください。

## STEP 6: 制約で領域を分ける

### やること

`layout.rs` の `Layout::split` を実装します。縦なら高さ、横なら幅を対象にし、`Length` と `Percentage` を順に割り当てます。残りを `Fill` の重みに応じて分け、整数の端数を最後の `Fill` に渡します。空の領域も返す個数は制約の個数と同じです。

### コード例/ポイント

```rust
// 高さ 24 を [Length(1), Fill(1), Length(1)] に分ける
// => 1 行 + 22 行 + 1 行
// 長さ 10 の [Fill(1), Fill(2), Fill(1)] => [2, 5, 3]
```

`demo.rs` の `render` はヘッダ・本体・フッタを分けたあと、各領域に Widget を描きます。画面サイズをハードコードしません。

### 確認方法

`cargo test -p p07_own_tui_starter layout::tests`。端末が狭くて `Length` が入りきらないときもパニックしないこと。

## STEP 7: raw 入力と端末の後片付けを確認する

### やること

`input.rs` の `parse_keys` と `parse_one` の TODO を埋めます。`Esc`、矢印の CSI (`ESC [ A`)、Backspace、Ctrl キー、UTF-8 を `Key` に変えます。`parse_escape` / `parse_utf8` を再利用します。`parse_one` が返す消費バイト数だけ進めます。`terminal.rs` の `RawMode` は `tcgetattr` で元の設定を保存し、`tcsetattr` で raw モードに入れ、Drop で戻します。`Events::next` は `poll` と `read`、`size` は `ioctl` を使います。

### コード例/ポイント

```rust
// b"a\x1b[A" => [Key::Char('a'), Key::Up]
// b"\x11" => [Key::Ctrl('q')]
```

端末 I/O は `unsafe` の薄い境界に閉じ込めます。`Terminal` は代替スクリーンを Drop で戻し、パニックフックも `restore` を呼びます。実際の `read` は UTF-8 / ESC シーケンス途中で分割される可能性があります。この教材の簡易パーサは 1 回の read 内で完結する前提なので、ストリーミング化は発展課題です。

### 確認方法

`cargo test -p p07_own_tui_starter input::tests`。PTY のある端末で最後にデモを動かし、終了後に入力エコーとカーソルが戻ることも確認します。

## STEP 8: デモを統合して動かす

### やること

`demo.rs` の `App::handle_key` と `App::render` を読み、状態更新と描画を分離している点を確認します。`main.rs` は 250ms ごとに Tick を受け取り、毎回新しい Buffer を作って `Terminal::draw` で差分を表示します。TODO をすべて埋めたらタスクを追加し、日本語入力・タブ移動・終了を試します。

### コード例/ポイント

```rust
// app.handle_key(key);  // 状態のみ変更
// terminal.draw(|buf| app.render(buf))?;  // 状態から画面を作る
```

この順序にすると `demo::tests` は端末を起動せずに実行できます。starter の `demo.rs` と `terminal.rs` は先に動く部品として提供しています。終了キーは `q` / Ctrl-C / Ctrl-D です。

### 確認方法

`cargo test -p p07_own_tui_starter` の全テストを通し、`cargo run -p p07_own_tui_starter` を実端末で起動します。`a`→日本語入力→Enter→Space→Tab→`q` を試し、終了後の端末が正常か確認します。模範解答側も `cargo test -p p07_own_tui` で比較できます。

## 発展課題

- `Events` に未処理バイトのバッファを持たせ、UTF-8 と CSI が複数回の `read` に分かれても復元する。
- カーソル移動・スタイル切替の回数を数え、全画面再描画と差分描画の出力バイト数を比較する。
- 結合文字・絵文字の ZWJ・可変幅など、教材の単純な `char_width` では表現できない文字列を調べる。
- `Widget` を一つ追加し、描画結果を `Buffer::line` で検査するテストを書く。
