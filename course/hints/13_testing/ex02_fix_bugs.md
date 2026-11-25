失敗したテストの名前と `left` / `right` を 1 つずつ見ていきましょう。
`left` が実際の値、`right` が期待値です。

- `moving_past_bottom_scrolls_minimally`: cursor=10, height=10 のとき offset は 1 になるべき。今の式では?
- `short_list_visible_range`: 項目が 3 個しかないのに `0..10` を返していないか?
- `page_down_stops_at_last_item`: cursor が `len` (= 範囲外) になっていないか?

---

`follow_cursor` の下方向: カーソルを **最下行** に置くとき、見えている範囲は
`offset..offset + height` で、最下行は `offset + height - 1`。これが `cursor` と等しいので
`offset = cursor + 1 - height` です (`cursor - height` は 1 ずれるうえ、cursor < height だと usize の引き算でパニックします)。

---

3 か所の修正:

```rust
self.offset = self.cursor + 1 - self.height;                 // follow_cursor
self.cursor = (self.cursor + self.height).min(self.len - 1); // page_down
self.offset..(self.offset + self.height).min(self.len)       // visible
```
