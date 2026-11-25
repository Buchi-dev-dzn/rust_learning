`Display` の `fmt` では、`write!(f, "...", ...)` で `Formatter` に文字列を書き込みます。
`write!` の戻り値はそのまま `fmt::Result` なので、最後の式にすれば OK です。

---

`Default` は `fn default() -> Self` を 1 つ持つトレイトです。

```rust
impl Default for Style {
    fn default() -> Self {
        Style { /* ... */ }
    }
}
```

トレイトのデフォルト実装は、`;` で終わっていたシグネチャに `{ ... }` の本体を付けるだけです。

---

```rust
// Size
write!(f, "{}x{}", self.w, self.h)

// Color
let name = match self {
    Color::Reset => "reset",
    Color::Red => "red",
    Color::Green => "green",
    Color::Blue => "blue",
};
write!(f, "{}", name)

// Describe
fn describe(&self) -> String {
    format!("<{}>", self.name())
}
```
