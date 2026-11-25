まず width と height がそれぞれ同じか確かめます。セル数では寸法の違いを検出できません。

---

`old.cells.iter().zip(&next.cells).enumerate()` で同じ位置のセルを比べます。`Cell` の `PartialEq` は文字と色の両方を比べます。

---

添字 `index` から `x = index % width`, `y = index / width` を計算し、`after.clone()` を `Change` に入れます。幅 0 ならセルは空なのでループ内の割り算は実行されません。
