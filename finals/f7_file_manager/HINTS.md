# F7 ヒント

まず `Rect` と `Buffer` を完成し、`cargo test -p f7_file_manager --test acceptance geometry_clamps_and_splits` から確かめよう。横分割では `min(left, width)`、縮小では `saturating_sub` を使う。セルの添字は `y * width + x`。

---

`text` では `chars()` を走査し、1 文字ずつ表示幅を加算する。右端に全角文字が半分しか入らなければそこで止める。`diff` は前フレームとセルを比較し、位置が変わったセルだけ 1 始まりの ANSI カーソル移動を出す。最後にスタイルを戻す。

---

`refresh` では `read_dir` の結果を全件集めてから `sort_by` する。比較は `b.is_dir.cmp(&a.is_dir)` → 小文字化した名前 → 元の名前。選択中の名前を記録してから並び替え、同じ名前がなければ最終 index に丸める。`down` は `min(selected + 1, len.saturating_sub(1))`、`up` は `saturating_sub(1)`。画面描画と `FileManager` の状態を混ぜず、端末なしで検証しよう。
