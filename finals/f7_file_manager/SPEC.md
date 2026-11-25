# F7: 自作フレームワークでファイルマネージャ

Stage 4 の仕上げとして、**libc 以外の外部クレートを使わず**、セルバッファ・差分描画・Widget・レイアウトから作る。実際のディレクトリを閲覧できる TUI にする。`src/lib.rs` は端末非依存の描画と状態、`src/main.rs` は termios・poll・端末サイズ・イベントループを担当する。受け入れテストは `lib.rs` のみを呼ぶ。`cargo run -p f7_file_manager -- [ディレクトリ]` で起動する (省略時はカレントディレクトリ)。

## 公開 API と境界条件

- `Rect::new(x,y,width,height)` はセル座標。`split_horizontal(left)` は左幅を自身の幅で頭打ちにし、右領域はその直後。`inner()` は上下左右 1 セル縮め、狭い領域の幅・高さは 0 に丸める。
- `Cell { ch, selected }` は空白・非選択が既定。`Buffer::new(width,height)` は空セルで埋める。`get(x,y)` は範囲外 `None`、`put(x,y,ch,selected)` は範囲外や制御文字を無視する。`line(y)` はその行を返し、範囲外は空文字。
- `display_width(ch)` は ASCII 1、代表的な CJK/絵文字 2、制御・結合文字 0。`Buffer::text(area,text,selected)` は area 内に最初の行を出し、画面右端でも切る。2 セル文字が半分だけ入る場合は描かない。2 セル目は空白セルとして確保する。
- `Buffer::diff(previous)` は同サイズなら変更セルだけを `ESC[{row};{col}H` (1 始まり) で移動して描く。選択セルは `ESC[7m`、通常セルは `ESC[0m`。末尾は必ず `ESC[0m`。サイズが変われば `ESC[2J` から全画面描画する。前フレーム自体は変更しない。
- `Widget::render(area,&mut buffer)` で描画する。`FileList { entries, selected, offset }` は offset 番目から area.height 件描く。ディレクトリは `▸`、ファイルは空白を先頭に付け、選択行のセルに selected を設定する。狭い領域ではクリップする。
- `Entry { name, is_dir }`。`FileManager::open(path)` はパスを canonicalize し、実際の `read_dir` から一覧を作る。開けなければ `io::Error`。`refresh()` はディレクトリを先に、名前の小文字化順 (同点なら元の名前順) で並べる。以前の選択項目が残れば名前と種類で選択を維持、消えたら最終 index に丸める。空なら 0。非 UTF-8 名は lossy 表示 (操作は表示名を使うため非 UTF-8 パスへの移動は保証しない)。
- `down/up` は末尾/先頭で止まり、空一覧では 0。`scroll(height)` は height 0 なら何もせず、選択が見える最小の offset へ移動する。`enter()` は選択がディレクトリなら移動して一覧を読み直し、ファイルなら `message` に `ファイル: 名前`、空なら何もしない。`parent()` は親へ移動し、元のディレクトリを選択する。ルートでは何もしない。読み取り失敗は `io::Error` とし現在状態を保つ。
- `render(&mut Buffer)` は上段にパス、中央に一覧と幅 40 以上なら詳細ペイン、最下段にキー操作とメッセージ。0×0 や狭い画面でもパニックしない。

## 端末 UI

`main.rs` は libc の `tcgetattr/tcsetattr` で raw モード、`ioctl(TIOCGWINSZ)` でサイズ、`poll/read` で入力を扱う。代替スクリーン・カーソル非表示を使い、通常終了・I/O エラー・パニックのいずれでも端末状態を戻す。`j/k` または上下キーで移動、Enter/`l`/右キーで開く、`h`/左キーで親、`r` で再読込、`q`/Ctrl-C/Ctrl-D で終了。読み取り失敗は画面のメッセージに表示する。フレームごとに `Buffer::diff` だけを送る。

## 合格条件

`cargo test -p f7_file_manager`。starter はコンパイルできるが受け入れテストが不合格。`course/finals/f7_file_manager/solution/src/` は解答、`starter/src/` はリセット用。テストを変更せず TODO を埋める。
