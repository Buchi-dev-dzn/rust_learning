`Termios` は `Copy` なので `let mut r = *t;` でコピーを作れます。
ビットを落とすのは `r.iflag &= !(A | B | C);`、立てるのは `r.cflag |= CS8;` です。
`cflag` は「`CSIZE` を落としてから `CS8` を立てる」の順番が大事です (文字サイズのビット群を入れ替える)。

---

`RawGuard::new` では、まず `let orig = backend.get_attr();` で保存し、`backend.set_attr(make_raw(&orig))`、
`backend.write_str(...)` をしてから `RawGuard { backend, orig }` を返します。

`Drop` では `self.backend.write_str(...)` と `self.backend.set_attr(self.orig)` を **開始と逆の順番** で呼びます。

---

```rust
fn make_raw(t: &Termios) -> Termios {
    let mut r = *t;
    r.iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
    r.oflag &= !OPOST;
    r.lflag &= !(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
    r.cflag &= !(CSIZE | PARENB);
    r.cflag |= CS8;
    r.vmin = 1;
    r.vtime = 0;
    r
}

impl<B: TermBackend> Drop for RawGuard<'_, B> {
    fn drop(&mut self) {
        self.backend.write_str("\x1b[?25h\x1b[?1049l");
        self.backend.set_attr(self.orig);
    }
}
```
