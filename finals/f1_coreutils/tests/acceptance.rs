//! F1 受け入れテスト (編集しないこと)

use f1_coreutils::*;
use std::path::PathBuf;
use std::process::Command;

fn temp_file(tag: &str, content: &[u8]) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("f1_coreutils_{}_{}", std::process::id(), tag));
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(format!("{tag}.txt"));
    std::fs::write(&p, content).unwrap();
    p
}

/// run をメモリ上で実行して (終了コード, stdout, stderr) を返す
fn run_mem(args: &[&str], stdin: &[u8]) -> (i32, Vec<u8>, String) {
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    let mut input = stdin;
    let mut out = Vec::new();
    let mut err = Vec::new();
    let code = run(&args, &mut input, &mut out, &mut err);
    (code, out, String::from_utf8(err).unwrap())
}

#[test]
fn count_basic() {
    let c = count(b"hello world\nfoo\n");
    assert_eq!(c, Counts { lines: 2, words: 3, chars: 16, bytes: 16 });
}

#[test]
fn count_edge_cases() {
    assert_eq!(count(b""), Counts::default());
    // 最後に改行がない行は lines に数えない
    let c = count(b"  a\t\tb \r\n\x0bc");
    assert_eq!((c.lines, c.words, c.bytes), (1, 3, 11));
}

#[test]
fn count_utf8_chars() {
    let c = count("日本語 テキスト\n".as_bytes());
    assert_eq!(c.chars, 9);
    assert_eq!(c.bytes, 23);
    assert_eq!(c.words, 2);
    // 不正な UTF-8 でもパニックしない
    let c = count(&[0xff, 0xfe, b'a']);
    assert_eq!(c.chars, 3);
}

#[test]
fn format_counts_default_and_flags() {
    let c = Counts { lines: 3, words: 5, chars: 18, bytes: 20 };
    assert_eq!(
        format_counts(&c, &WcFlags::default(), Some("a.txt")),
        "       3       5      20 a.txt"
    );
    let f = WcFlags { lines: true, ..Default::default() };
    assert_eq!(format_counts(&c, &f, None), "       3");
    // 順序は常に lines, words, chars, bytes
    let f = WcFlags { bytes: true, chars: true, ..Default::default() };
    assert_eq!(format_counts(&c, &f, None), "      18      20");
}

#[test]
fn head_lines_cases() {
    let input = b"1\n2\n3\n";
    assert_eq!(head_lines(input, 2), b"1\n2\n");
    assert_eq!(head_lines(input, 10), input);
    assert_eq!(head_lines(input, 0), b"");
    assert_eq!(head_lines(b"a\nb", 2), b"a\nb");
}

#[test]
fn number_lines_cases() {
    assert_eq!(number_lines(b"a\nb", 1), (b"     1\ta\n     2\tb".to_vec(), 3));
    assert_eq!(number_lines(b"", 5), (Vec::new(), 5));
    assert_eq!(number_lines(b"\n\n", 9), (b"     9\t\n    10\t\n".to_vec(), 11));
}

#[test]
fn run_wc_stdin() {
    let (code, out, _) = run_mem(&["wc"], b"hello world\nfoo\n");
    assert_eq!(code, 0);
    assert_eq!(out, b"       2       3      16\n");
    let (_, out, _) = run_mem(&["wc", "-lw"], b"a b\n");
    assert_eq!(out, b"       1       2\n");
}

#[test]
fn run_wc_files_and_total() {
    let a = temp_file("wc_a", b"one two\n");
    let b = temp_file("wc_b", b"x\ny\nz\n");
    let (a, b) = (a.to_str().unwrap(), b.to_str().unwrap());
    let (code, out, _) = run_mem(&["wc", "-l", a, b], b"");
    assert_eq!(code, 0);
    let expected = format!("       1 {a}\n       3 {b}\n       4 total\n");
    assert_eq!(String::from_utf8(out).unwrap(), expected);
}

#[test]
fn run_head_default_and_n() {
    let input: String = (1..=15).map(|i| format!("{i}\n")).collect();
    let (code, out, _) = run_mem(&["head"], input.as_bytes());
    assert_eq!(code, 0);
    assert_eq!(String::from_utf8(out).unwrap().lines().count(), 10);
    let (_, out, _) = run_mem(&["head", "-n", "3"], input.as_bytes());
    assert_eq!(out, b"1\n2\n3\n");
    let (_, out, _) = run_mem(&["head", "-n2"], input.as_bytes());
    assert_eq!(out, b"1\n2\n");
}

#[test]
fn run_head_multiple_files_headers() {
    let a = temp_file("head_a", b"a1\na2\n");
    let b = temp_file("head_b", b"b1\nb2\n");
    let (a, b) = (a.to_str().unwrap(), b.to_str().unwrap());
    let (code, out, _) = run_mem(&["head", "-n", "1", a, b], b"");
    assert_eq!(code, 0);
    let expected = format!("==> {a} <==\na1\n\n==> {b} <==\nb1\n");
    assert_eq!(String::from_utf8(out).unwrap(), expected);
}

#[test]
fn run_head_invalid_number() {
    let (code, out, err) = run_mem(&["head", "-n", "abc"], b"x\n");
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.contains("invalid number of lines: abc"), "stderr: {err}");
}

#[test]
fn run_cat_numbering_continues_across_files() {
    let a = temp_file("cat_a", b"x\ny\n");
    let (code, out, _) = run_mem(&["cat", "-n", a.to_str().unwrap(), "-"], b"z\n");
    assert_eq!(code, 0);
    assert_eq!(out, b"     1\tx\n     2\ty\n     3\tz\n");
    let (_, out, _) = run_mem(&["cat"], b"raw \xff bytes");
    assert_eq!(out, b"raw \xff bytes");
}

#[test]
fn run_missing_file_continues_and_exit_1() {
    let a = temp_file("missing_ok", b"ok\n");
    let missing = "/nonexistent/f1_coreutils/nope.txt";
    let (code, out, err) = run_mem(&["cat", missing, a.to_str().unwrap()], b"");
    assert_eq!(code, 1);
    assert_eq!(out, b"ok\n");
    assert!(err.starts_with(&format!("f1_coreutils: {missing}:")), "stderr: {err}");
}

#[test]
fn run_usage_and_invalid_option() {
    let (code, _, err) = run_mem(&[], b"");
    assert_eq!(code, 2);
    assert!(err.starts_with("usage: f1_coreutils"));
    let (code, _, err) = run_mem(&["frobnicate"], b"");
    assert_eq!(code, 2);
    assert!(err.starts_with("usage: f1_coreutils"));
    let (code, out, err) = run_mem(&["wc", "-x"], b"abc");
    assert_eq!(code, 2);
    assert!(out.is_empty());
    assert!(err.contains("invalid option: -x"), "stderr: {err}");
}

#[test]
fn binary_exit_codes_and_output() {
    let exe = env!("CARGO_BIN_EXE_f1_coreutils");
    let a = temp_file("bin_a", b"l1\nl2\nl3\n");

    let out = Command::new(exe).args(["wc", "-l"]).arg(&a).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), format!("       3 {}\n", a.display()));

    let out = Command::new(exe).args(["head", "-n", "1"]).arg(&a).output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(out.stdout, b"l1\n");

    let out = Command::new(exe).arg("nope").output().unwrap();
    assert_eq!(out.status.code(), Some(2));

    let out = Command::new(exe).args(["cat", "/nonexistent/f1/x"]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stderr.is_empty());
}
