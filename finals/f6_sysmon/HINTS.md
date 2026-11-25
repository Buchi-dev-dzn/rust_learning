# F6: htop 風システムモニタ — 段階ヒント

まず `SPEC.md` と `tests/acceptance.rs` を読み、`src/lib.rs` の小さい関数から順に埋めよう。テストは編集しない。`cargo test -p f6_sysmon` で進捗を確認できる。

## ヒント 1: データの取り込み

`Snapshot::total_cpu` は `iter().map(...).sum()` で求められる。`parse_ps` は各行の先頭 3 フィールドだけを切り出し、残りを *まとめて* command にする。`split_whitespace()` ですべて分割してしまうと、空白を含むコマンド名が壊れる。ヘッダや不正な数値は `parse::<T>().ok()` などで飛ばそう。

`History` は `VecDeque` の先頭を `pop_front()`、末尾を `push_back()` する。容量 0 の場合は push 前に戻る。

---

## ヒント 2: 表示リストと選択

`visible()` では `filter.to_lowercase()` と `command.to_lowercase()` を比較し、残った参照を `sort_by` する。比較キーが同値なら、降順の反転を **先に** 済ませてから `pid` 昇順で同値を解決する。

`update`、`set_sort`、`set_filter` に共通の補助関数を用意しよう。変更前に選択中の PID と index を保存し、変更後の `visible()` から PID を探す。見つからなければ `old_index.min(visible.len().saturating_sub(1))`。空リストでも index は 0 にできる。

---

## ヒント 3: 入力・画面

上下・PageUp/Down・Home/End はモード共通なので、`handle_input` の先頭で処理する。Filter モードでは `q` や `/` も検索文字列であり、Backspace は `String::pop()` で最後の *文字* を削除する。Normal モードの Esc はフィルタ解除、一時停止中の `update` は履歴も変えない。

`main.rs` の `ui` では `Layout::vertical` で画面をグラフ・表・ステータスの 3 領域に分ける。表の表示行数は領域の高さから枠 2 行とヘッダ 1 行を引き、`scroll_offset` から必要行だけ `Row` に変換する。選択行に `Modifier::REVERSED`、ソート対象の見出しに `▼` / `▲` を付ける。端末 I/O は `main.rs` に留め、状態のテストは `lib.rs` の `Snapshot` 注入で行おう。
