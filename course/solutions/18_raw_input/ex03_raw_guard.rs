//! # raw モードと Drop ガード (偽の端末で)
//!
//! 本物の termios を触るとテストできないので、termios を真似た `Termios` 構造体と、
//! 端末を抽象化した `TermBackend` トレイトを用意した。次の 2 つを実装しよう。
//!
//! 1. `make_raw(t)` … C の `cfmakeraw` と同じ加工をしたコピーを返す
//!    - `iflag` から `IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON` を落とす
//!    - `oflag` から `OPOST` を落とす
//!    - `lflag` から `ECHO | ECHONL | ICANON | ISIG | IEXTEN` を落とす
//!    - `cflag` から `CSIZE | PARENB` を落としてから `CS8` を立てる
//!    - `vmin = 1`, `vtime = 0`
//!    - 上に書いていないビットはそのまま残す
//! 2. `RawGuard` … `RawGuard::new(backend)` で
//!    「現在の設定を保存 → raw に設定 → `"\x1b[?1049h\x1b[?25l"` を書く」。
//!    Drop されたら「`"\x1b[?25h\x1b[?1049l"` を書く → 保存した設定に戻す」。
//!    パニックで抜けても元に戻ることをテストで確かめる。
//!
//! - 関連: 第18章「libc で raw モードに入る」「Drop ガードで必ず元に戻す」
//!
//! テスト (`mod tests`) は編集しないこと。

// termios のフラグ (値は macOS/Linux と同じである必要はない。ここでは練習用)
const IGNBRK: u32 = 1 << 0;
const BRKINT: u32 = 1 << 1;
const PARMRK: u32 = 1 << 3;
const ISTRIP: u32 = 1 << 5;
const INLCR: u32 = 1 << 6;
const IGNCR: u32 = 1 << 7;
const ICRNL: u32 = 1 << 8;
const IXON: u32 = 1 << 10;
const IUTF8: u32 = 1 << 14;

const OPOST: u32 = 1 << 0;

const ECHO: u32 = 1 << 3;
const ECHONL: u32 = 1 << 4;
const ICANON: u32 = 1 << 8;
const ISIG: u32 = 1 << 7;
const IEXTEN: u32 = 1 << 10;

const CSIZE: u32 = 0b11 << 8;
const CS7: u32 = 0b10 << 8;
const CS8: u32 = 0b11 << 8;
const PARENB: u32 = 1 << 12;
const CREAD: u32 = 1 << 11;

#[derive(Clone, Copy, Debug, PartialEq)]
struct Termios {
    iflag: u32,
    oflag: u32,
    cflag: u32,
    lflag: u32,
    vmin: u8,
    vtime: u8,
}

trait TermBackend {
    fn get_attr(&self) -> Termios;
    fn set_attr(&mut self, t: Termios);
    fn write_str(&mut self, s: &str);
}

fn make_raw(t: &Termios) -> Termios {
    let mut r = *t; // Copy なのでコピーが作られる
    r.iflag &= !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON);
    r.oflag &= !OPOST;
    r.lflag &= !(ECHO | ECHONL | ICANON | ISIG | IEXTEN);
    r.cflag &= !(CSIZE | PARENB);
    r.cflag |= CS8;
    r.vmin = 1;
    r.vtime = 0;
    r
}

struct RawGuard<'a, B: TermBackend> {
    backend: &'a mut B,
    orig: Termios,
}

impl<'a, B: TermBackend> RawGuard<'a, B> {
    fn new(backend: &'a mut B) -> Self {
        let orig = backend.get_attr();
        backend.set_attr(make_raw(&orig));
        backend.write_str("\x1b[?1049h\x1b[?25l");
        RawGuard { backend, orig }
    }

    /// ガードを通して端末に書く
    fn backend(&mut self) -> &mut B {
        self.backend
    }
}

impl<B: TermBackend> Drop for RawGuard<'_, B> {
    fn drop(&mut self) {
        // 開始と逆の順番で戻す
        self.backend.write_str("\x1b[?25h\x1b[?1049l");
        self.backend.set_attr(self.orig);
    }
}

// ---- テスト用の偽の端末 ----
struct FakeTerm {
    current: Termios,
    history: Vec<Termios>,
    output: String,
}

impl FakeTerm {
    fn new() -> Self {
        FakeTerm {
            current: Termios {
                iflag: ICRNL | IXON | BRKINT | IUTF8,
                oflag: OPOST,
                cflag: CS7 | PARENB | CREAD,
                lflag: ECHO | ICANON | ISIG | IEXTEN,
                vmin: 0,
                vtime: 5,
            },
            history: Vec::new(),
            output: String::new(),
        }
    }
}

impl TermBackend for FakeTerm {
    fn get_attr(&self) -> Termios {
        self.current
    }
    fn set_attr(&mut self, t: Termios) {
        self.current = t;
        self.history.push(t);
    }
    fn write_str(&mut self, s: &str) {
        self.output.push_str(s);
    }
}

fn main() {
    let mut term = FakeTerm::new();
    {
        let mut g = RawGuard::new(&mut term);
        g.backend().write_str("hello");
    }
    println!("{:?}", term.output);
    println!("restored lflag = {:#x}", term.current.lflag);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_raw_clears_flags() {
        let orig = FakeTerm::new().current;
        let r = make_raw(&orig);
        assert_eq!(r.iflag, IUTF8); // 関係ないビットは残る
        assert_eq!(r.oflag, 0);
        assert_eq!(r.lflag, 0);
        assert_eq!(r.cflag, CS8 | CREAD);
        assert_eq!((r.vmin, r.vtime), (1, 0));
        // 元は変更されない
        assert_eq!(orig.lflag, ECHO | ICANON | ISIG | IEXTEN);
    }

    #[test]
    fn make_raw_keeps_unrelated_bits() {
        let t = Termios { iflag: !0, oflag: !0, cflag: 0, lflag: !0, vmin: 9, vtime: 9 };
        let r = make_raw(&t);
        assert_eq!(r.iflag, !(IGNBRK | BRKINT | PARMRK | ISTRIP | INLCR | IGNCR | ICRNL | IXON));
        assert_eq!(r.oflag, !OPOST);
        assert_eq!(r.lflag, !(ECHO | ECHONL | ICANON | ISIG | IEXTEN));
        assert_eq!(r.cflag, CS8);
    }

    #[test]
    fn guard_enables_and_restores() {
        let mut term = FakeTerm::new();
        let orig = term.current;
        {
            let mut g = RawGuard::new(&mut term);
            g.backend().write_str("X");
            assert_eq!(g.backend().current, make_raw(&orig));
        }
        assert_eq!(term.current, orig);
        assert_eq!(term.history, vec![make_raw(&orig), orig]);
        assert_eq!(term.output, "\x1b[?1049h\x1b[?25lX\x1b[?25h\x1b[?1049l");
    }

    #[test]
    fn guard_restores_on_panic() {
        let mut term = FakeTerm::new();
        let orig = term.current;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _g = RawGuard::new(&mut term);
            panic!("アプリ内でパニック");
        }));
        assert!(result.is_err());
        assert_eq!(term.current, orig);
        assert!(term.output.ends_with("\x1b[?25h\x1b[?1049l"));
    }
}
