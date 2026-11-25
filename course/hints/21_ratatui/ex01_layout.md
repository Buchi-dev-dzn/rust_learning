まず `top = header.min(area.height)` とします。残りの高さから `bottom` を決めます。

---

`middle = area.height - top - bottom`。本文の y は元の y + top、フッターは元の y + top + middle です。

---

`u16` の足し算は端末の座標上限であふれることがあります。`saturating_add` を使い、幅・x は `..area` で引き継ぎます。
