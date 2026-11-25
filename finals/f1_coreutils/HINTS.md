まずは純粋な関数 (`count`, `format_counts`, `head_lines`, `number_lines`) から作ろう。
これらは I/O を一切しないので、テストを 1 つずつ通しながら進められる。

- `count` は「今、単語の中にいるか」を表す `bool` を 1 つ持ってバイト列を 1 回なめるだけで書ける。
- `chars` は「UTF-8 の先頭バイトの数」。`b & 0xC0 != 0x80` のバイトを数えればよい。
- `number_lines` には `input.split_inclusive(|&b| b == b'\n')` が便利 (改行を含めたまま分割してくれる)。

`cargo test -p f1_coreutils count` のようにテスト名で絞り込んで実行できる。

---

`run` は「サブコマンドで分岐 → オプションとファイル名を分ける → 入力ごとに処理」の 3 段に分けると見通しがよい。

```rust
pub fn run(args: &[String], stdin: &mut dyn Read, stdout: &mut dyn Write, stderr: &mut dyn Write) -> i32 {
    let Some(cmd) = args.first() else { /* usage を出して */ return 2; };
    match cmd.as_str() {
        "wc" => cmd_wc(&args[1..], stdin, stdout, stderr),
        "head" => cmd_head(&args[1..], stdin, stdout, stderr),
        "cat" => cmd_cat(&args[1..], stdin, stdout, stderr),
        _ => { /* usage */ 2 }
    }
}
```

入力を読むヘルパーを 1 つ作っておくと 3 コマンドで使い回せる:

```rust
fn read_source(path: &str, stdin: &mut dyn Read) -> std::io::Result<Vec<u8>> {
    let mut buf = Vec::new();
    if path == "-" { stdin.read_to_end(&mut buf)?; }
    else { std::fs::File::open(path)?.read_to_end(&mut buf)?; }
    Ok(buf)
}
```

ファイル引数が空なら `vec!["-".to_string()]` として扱えば、標準入力の場合を特別扱いしなくて済む。

---

エラー時の振る舞いを整理しよう。

- 未知のオプション / 不正な `-n` の値 → **何も処理せず**すぐに `return 2`。
- ファイルが開けない → `writeln!(stderr, "f1_coreutils: {path}: {e}")` して `code = 1` にし、`continue` で次のファイルへ。
- `head -n` は `-n 3` (次の引数が値) と `-n3` (同じ引数内) の両方を受ける。
  `a.strip_prefix("-n")` の結果が空文字列かどうかで見分けられる。
- `cat -n` の番号をファイルをまたいで続けるには、`number_lines` の戻り値の「次の行番号」を次の呼び出しの `start` に渡す。
- `head` の複数ファイルでは「これまでに出力したファイル数」を数え、2 つ目以降のヘッダの前にだけ `\n` を書く。
