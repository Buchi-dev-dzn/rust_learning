# 第18章 raw モードとキー入力の解析

## この章のゴール

- canonical (cooked) モードと raw モードの違いを説明できる
- termios の主なフラグと、`tcgetattr` / `cfmakeraw` / `tcsetattr` の役割がわかる
- Drop ガードで「必ず端末設定を元に戻す」仕組みを書ける
- stdin からバイトを読み、矢印キーなどのエスケープシーケンスを `Key` enum に解析できる
- `ioctl(TIOCGWINSZ)` で端末サイズを取れることを知る

この章の内容は、プロジェクト p03 (ライブラリなしでターミナル制御) と卒業課題 F3 (キー入力パーサ)
の直接の準備です。crossterm が内部でやっていることを、自分の手で一度やってみましょう。

## canonical モード (普段の状態)

シェルで `cat` を実行して文字を打ってみると、次のことに気づきます。

- 入力した文字が画面に表示される (**エコー**)
- Backspace で消せる (**行編集**)
- Enter を押すまで `cat` には何も届かない (**行バッファリング**)
- `Ctrl-C` でプロセスが終了する (**シグナル生成**)

これらはアプリではなく、カーネルの **line discipline** (第17章) がやっています。
この標準の状態を **canonical モード** (cooked モードとも) と呼びます。
普通の CLI には便利ですが、TUI には全部じゃまです。

- `j` を押した瞬間にカーソルを動かしたいのに、Enter まで届かない
- 押したキーが勝手に画面に表示されて描画が崩れる
- `Ctrl-C` や `Ctrl-Z` を自分で処理したい (エディタでコピーに使う等)

## raw モード

**raw モード** は、line discipline の加工をほぼすべて止めた状態です。

- キーを押すたびに、そのバイトがすぐアプリに届く
- エコーしない
- `Ctrl-C` は SIGINT にならず、バイト `0x03` として届く
- 出力の `\n` → `\r\n` 変換も止まる (だから TUI では `\r\n` を書くか、カーソルを明示移動する)

これらの設定は **termios** という構造体で管理されています。主なフラグ:

| フィールド | フラグ | 意味 (raw ではオフ) |
|---|---|---|
| `c_lflag` | `ECHO` | 入力をエコーする |
| `c_lflag` | `ICANON` | canonical モード (行バッファ) |
| `c_lflag` | `ISIG` | `Ctrl-C`/`Ctrl-Z` でシグナルを送る |
| `c_lflag` | `IEXTEN` | `Ctrl-V` などの拡張入力処理 |
| `c_iflag` | `IXON` | `Ctrl-S`/`Ctrl-Q` でフロー制御 |
| `c_iflag` | `ICRNL` | 入力の `\r` を `\n` に変換 (Enter が `\n` になる理由) |
| `c_oflag` | `OPOST` | 出力の後処理 (`\n` → `\r\n`) |
| `c_cc[VMIN]` | - | `read` が返るまでの最小バイト数 |
| `c_cc[VTIME]` | - | `read` のタイムアウト (0.1 秒単位) |

`VMIN = 1, VTIME = 0` なら「1 バイト来るまで待つ」、`VMIN = 0, VTIME = 1` なら
「最大 0.1 秒待って、来なければ 0 バイトで返る」になります。後者はアニメーションのある
TUI で、入力がなくても定期的に処理を回したいときに使います。

## libc で raw モードに入る

標準ライブラリには termios の API がないので、`libc` クレートで C の関数を直接呼びます。
(このコードは外部クレートを使い、実際の端末も必要なので、実行ボタンはありません。プロジェクト p03 で動かします。)

```rust,noplayground
use std::io;
use std::os::fd::AsRawFd;

fn enable_raw_mode() -> io::Result<libc::termios> {
    let fd = io::stdin().as_raw_fd();
    unsafe {
        // 1. 現在の設定を取得 (後で戻すために保存する)
        let mut orig: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(fd, &mut orig) != 0 {
            return Err(io::Error::last_os_error());
        }
        // 2. コピーを raw 用に書き換える
        let mut raw = orig;
        libc::cfmakeraw(&mut raw); // ECHO, ICANON, ISIG, IXON, ICRNL, OPOST... をまとめてオフ
        raw.c_cc[libc::VMIN] = 1;
        raw.c_cc[libc::VTIME] = 0;
        // 3. 反映する。TCSAFLUSH = 未読の入力を捨ててから切り替え
        if libc::tcsetattr(fd, libc::TCSAFLUSH, &raw) != 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(orig)
    }
}

fn disable_raw_mode(orig: &libc::termios) {
    let fd = io::stdin().as_raw_fd();
    unsafe {
        libc::tcsetattr(fd, libc::TCSAFLUSH, orig);
    }
}
```

`cfmakeraw` を使わずに自分でフラグを落とす場合は、次のようにビット演算をします。

```rust,noplayground
raw.c_iflag &= !(libc::BRKINT | libc::ICRNL | libc::INPCK | libc::ISTRIP | libc::IXON);
raw.c_oflag &= !(libc::OPOST);
raw.c_cflag |= libc::CS8;
raw.c_lflag &= !(libc::ECHO | libc::ICANON | libc::IEXTEN | libc::ISIG);
```

`unsafe` が必要なのは、C の関数は Rust のコンパイラが安全性を検査できないからです。
ポインタの有効性などをプログラマが保証する、という宣言です。

## Drop ガードで必ず元に戻す

raw モードのまま終了すると、シェルが「エコーなし・改行がおかしい」状態で残ります。
`return` でも `?` でもパニックでも確実に戻すために、**Drop ガード** を使います。

```rust,noplayground
struct RawMode {
    orig: libc::termios,
}

impl RawMode {
    fn enable() -> io::Result<Self> {
        Ok(RawMode { orig: enable_raw_mode()? })
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        disable_raw_mode(&self.orig);
    }
}

fn main() -> io::Result<()> {
    let _raw = RawMode::enable()?; // _raw がスコープを抜けると自動で元に戻る
    run()?;                        // ここでエラーやパニックが起きても戻る
    Ok(())
}
```

注意点:

- `let _ = RawMode::enable()?;` と書くと **その場で Drop される** ので意味がありません。
  `_raw` のように名前を付けた変数に束縛します。
- `std::process::exit()` は Drop を実行しません (第15章)。
- パニック時に Drop が走るのは、パニックが unwind する場合です (既定の設定)。
  さらに確実にするには、第19章で扱うパニックフックを併用します。

Drop ガードの考え方自体は標準ライブラリだけで試せます。

```rust
struct Guard(&'static str);

impl Drop for Guard {
    fn drop(&mut self) {
        println!("[{}] 後片付け: 端末設定を元に戻した (つもり)", self.0);
    }
}

fn work(fail: bool) -> Result<(), String> {
    let _g = Guard(if fail { "失敗ケース" } else { "成功ケース" });
    println!("raw モードで作業中...");
    if fail {
        return Err("途中でエラー".into()); // ここで return しても Drop は走る
    }
    Ok(())
}

fn main() {
    println!("{:?}", work(false));
    println!("{:?}", work(true));
}
```

## バイトを読む

raw モードでは、`stdin().read()` で届いたバイトをそのまま読みます。
`lines()` や `read_line()` は改行を待ってしまうので使いません。

```rust,noplayground
use std::io::Read;

let mut stdin = std::io::stdin();
let mut buf = [0u8; 32];
loop {
    let n = stdin.read(&mut buf)?; // 1 回のキー入力で 1〜数バイト
    for &b in &buf[..n] {
        print!("{b:#04x} ");
    }
    print!("\r\n");
    if buf[..n].contains(&b'q') {
        break;
    }
}
```

これを raw モードで動かして色々なキーを押すと、次のようなバイトが見えます。

| キー | バイト列 | 備考 |
|---|---|---|
| `a` | `61` | ASCII そのまま |
| `あ` (IME 確定) | `E3 81 82` | UTF-8 |
| Enter | `0D` | `\r` (ICRNL がオフなので `\n` にならない) |
| Tab | `09` | = Ctrl-I |
| Backspace | `7F` | 端末によっては `08` (= Ctrl-H) |
| Esc | `1B` | 単独 |
| Ctrl-A 〜 Ctrl-Z | `01` 〜 `1A` | `文字 & 0x1F` |
| ↑ ↓ → ← | `1B 5B 41` 〜 `44` | `ESC [ A` 〜 `ESC [ D` |
| Home / End | `1B 5B 48` / `1B 5B 46` | `ESC [ H` / `ESC [ F` (端末により `ESC [ 1 ~` / `ESC [ 4 ~`) |
| Delete | `1B 5B 33 7E` | `ESC [ 3 ~` |
| PageUp / PageDown | `ESC [ 5 ~` / `ESC [ 6 ~` | |
| F1〜F4 | `ESC O P` 〜 `ESC O S` | SS3 形式 |
| Shift+↑ など | `ESC [ 1 ; 2 A` | 引数 2 = Shift, 3 = Alt, 5 = Ctrl |
| Alt+x | `1B 78` | ESC + 文字 |

### Ctrl キーの仕組み

Ctrl を押しながら文字を打つと、端末はその文字の下位 5 ビットだけを送ります。
`'a'` は `0x61 = 0b0110_0001`、`& 0x1F` すると `0x01`。だから Ctrl-A は `0x01`、
Ctrl-C は `0x03` です。逆に `0x01..=0x1A` を受け取ったら `(b + b'a' - 1) as char` で
どの文字との組み合わせかがわかります。

Enter (`0x0D` = Ctrl-M)、Tab (`0x09` = Ctrl-I)、Backspace の一部 (`0x08` = Ctrl-H) が
Ctrl キーと区別できないのは、この仕組みのためです。

## Esc キー問題

Esc キー単独は `1B` の 1 バイト、矢印キーは `1B 5B 41` です。では `1B` を受け取った瞬間、
これは Esc キーなのか、矢印キーの始まりなのか?

答えは「続きが **すぐに** 来るかどうか」で判断します。端末は 1 つのキーのシーケンスを
まとめて送るので、矢印キーなら 3 バイトが同じ `read` で届くのが普通です。
実装としては:

1. `read` で届いたバイト列をまとめて解析する
2. バッファが `1B` で終わっていたら、少し (例: 50ms) 待って続きが来なければ Esc とみなす

vim の `ttimeoutlen` 設定がまさにこの待ち時間です。

## キーパーサを状態機械で書く

バイト列を `Key` に変換する処理は、**状態機械 (ステートマシン)** で書くと見通しがよくなります。

```
          ┌──── 通常の文字 → Key::Char / Ctrl / Enter ...
 Ground ──┤
          └── 0x1B → Escape ──┬── '[' → Csi ── 数字/';' を貯める ── 終端 (A,B,~...) → Key を確定
                               ├── 'O' → Ss3 ── 'P'..'S' → F1..F4
                               └── その他 → Alt(文字)
```

次の例は、よく使うキーだけを扱う最小のパーサです (演習ではもっと完全なものを作ります)。

```rust
#[derive(Debug, PartialEq)]
enum Key {
    Char(char),
    Ctrl(char),
    Enter,
    Backspace,
    Esc,
    Up,
    Down,
    Right,
    Left,
    Unknown(Vec<u8>),
}

fn parse(bytes: &[u8]) -> Vec<Key> {
    let mut keys = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            0x1b if bytes.get(i + 1) == Some(&b'[') && i + 2 < bytes.len() => {
                let k = match bytes[i + 2] {
                    b'A' => Key::Up,
                    b'B' => Key::Down,
                    b'C' => Key::Right,
                    b'D' => Key::Left,
                    _ => Key::Unknown(bytes[i..i + 3].to_vec()),
                };
                keys.push(k);
                i += 3;
                continue;
            }
            0x1b => keys.push(Key::Esc),
            b'\r' => keys.push(Key::Enter),
            0x7f => keys.push(Key::Backspace),
            0x01..=0x1a => keys.push(Key::Ctrl((b - 1 + b'a') as char)),
            0x20..=0x7e => keys.push(Key::Char(b as char)),
            _ => keys.push(Key::Unknown(vec![b])),
        }
        i += 1;
    }
    keys
}

fn main() {
    let input = b"hi\x1b[A\x1b[D\r\x03\x7f\x1b";
    for k in parse(input) {
        println!("{k:?}");
    }
}
```

これを UTF-8 (複数バイトの文字) や `ESC [ 3 ~`・`ESC [ 1 ; 5 C` のような引数付きシーケンスにも
対応させるのが演習 ex02 です。

## 端末サイズを取る

```rust,noplayground
fn terminal_size() -> Option<(u16, u16)> {
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            Some((ws.ws_col, ws.ws_row)) // (幅, 高さ)
        } else {
            None
        }
    }
}
```

リサイズ時には `SIGWINCH` が届きます。ライブラリなしで対応するには、シグナルハンドラでフラグを立て、
メインループで `terminal_size()` を取り直して再描画します。crossterm はこれを
`Event::Resize` として届けてくれます (第19章)。

## よくあるコンパイルエラーと読み方

**`error[E0133]: call to unsafe function is unsafe and requires unsafe block`**
→ `libc::tcgetattr` などの C 関数は `unsafe { ... }` の中で呼びます。Edition 2024 では
`unsafe fn` の中でも、unsafe な操作は `unsafe {}` ブロックで囲む必要があります。

**`error[E0599]: no method named `as_raw_fd` found for struct `Stdin``**
→ `use std::os::fd::AsRawFd;` が必要です (トレイトのメソッド)。

**`error[E0308]: mismatched types ... expected `u8`, found `char``**
→ `b'q'` (u8) と `'q'` (char) を混同しています。バイト列を扱うときは `b'...'` を使います。

**`error[E0716]` / ガードがすぐに Drop される**
→ `let _ = RawMode::enable()?;` はその場で Drop します。`let _raw = ...` にしましょう。
これはコンパイルエラーにならない **論理バグ** なので特に注意。

## まとめ

- canonical モードはカーネルが行編集・エコー・シグナルを担当する。TUI は raw モードにする
- termios を `tcgetattr` で保存 → `cfmakeraw` で加工 → `tcsetattr` で反映。戻すときは保存した値を再設定
- 元に戻す処理は Drop ガードに書く (`exit` を使わない、`let _ =` にしない)
- キー入力はバイト列。Ctrl は `& 0x1F`、特殊キーは `ESC [ ...` のエスケープシーケンス
- 解析は状態機械で。Esc 単独と矢印キーの区別には「続きがすぐ来るか」を使う

## 次へ

ここまでを手で書くと、端末ごとの差異 (Windows も含む) への対応が大変です。第19章では crossterm を使い、
raw モード・イベント・リサイズ・代替スクリーンをクロスプラットフォームに扱います。
