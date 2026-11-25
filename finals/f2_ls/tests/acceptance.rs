//! F2 受け入れテスト (編集しないこと)

use f2_ls::*;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn files(names: &[&str]) -> Vec<Entry> {
    names.iter().map(|n| Entry::new(n, Kind::File)).collect()
}

fn fixture(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("f2_ls_{}_{}", std::process::id(), tag));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("a.txt"), b"hello").unwrap();
    std::fs::write(dir.join("run.sh"), b"#!/bin/sh\n").unwrap();
    std::fs::set_permissions(dir.join("run.sh"), std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::set_permissions(dir.join("a.txt"), std::fs::Permissions::from_mode(0o644)).unwrap();
    std::fs::write(dir.join(".hidden"), b"").unwrap();
    std::os::unix::fs::symlink("a.txt", dir.join("link")).unwrap();
    dir
}

#[test]
fn entry_new_defaults() {
    let e = Entry::new("x", Kind::Dir);
    assert_eq!((e.mode, e.size, e.nlink, e.link_target), (0o755, 0, 1, None));
    assert_eq!(Entry::new("x", Kind::File).mode, 0o644);
    assert_eq!(Entry::new("x", Kind::Symlink).mode, 0o777);
}

#[test]
fn widths() {
    assert_eq!(char_width('a'), 1);
    assert_eq!(char_width('あ'), 2);
    assert_eq!(char_width('漢'), 2);
    assert_eq!(char_width('Ａ'), 2); // 全角英字
    assert_eq!(char_width('ｱ'), 1); // 半角カナ
    assert_eq!(char_width('\u{0301}'), 0); // 結合文字
    assert_eq!(char_width('\t'), 0);
    assert_eq!(str_width("日本語.txt"), 10);
    assert_eq!(str_width(""), 0);
}

#[test]
fn pad_right_uses_display_width() {
    assert_eq!(pad_right("ab", 5), "ab   ");
    assert_eq!(pad_right("日本", 6), "日本  ");
    assert_eq!(pad_right("toolong", 3), "toolong");
}

#[test]
fn colorize_by_kind() {
    assert_eq!(colorize(&Entry::new("d", Kind::Dir)), "\x1b[1;34md\x1b[0m");
    assert_eq!(colorize(&Entry::new("x", Kind::Executable)), "\x1b[1;32mx\x1b[0m");
    assert_eq!(colorize(&Entry::new("l", Kind::Symlink)), "\x1b[1;36ml\x1b[0m");
    assert_eq!(colorize(&Entry::new("f", Kind::File)), "f");
}

#[test]
fn grid_spec_example() {
    let e = files(&["a", "bb", "ccc", "dddd", "eeeee"]);
    assert_eq!(format_grid(&e, 20, false), vec!["a   ccc   eeeee", "bb  dddd"]);
}

#[test]
fn grid_single_row_and_single_column() {
    let e = files(&["a", "bb", "ccc"]);
    assert_eq!(format_grid(&e, 80, false), vec!["a  bb  ccc"]);
    // 幅が足りなければ 1 列 (行末に空白なし)
    assert_eq!(format_grid(&e, 3, false), vec!["a", "bb", "ccc"]);
    // 1 列でも収まらない場合も 1 列で出す
    assert_eq!(format_grid(&e, 1, false), vec!["a", "bb", "ccc"]);
    assert!(format_grid(&[], 80, false).is_empty());
}

#[test]
fn grid_exact_fit() {
    // 1+2+2+2+3 = 10 ちょうどで 3 列に収まる
    let e = files(&["a", "bb", "ccc"]);
    assert_eq!(format_grid(&e, 10, false), vec!["a  bb  ccc"]);
    assert_eq!(format_grid(&e, 9, false), vec!["a   ccc", "bb"]);
}

#[test]
fn grid_wide_chars() {
    let e = files(&["日本", "a", "かな", "b"]);
    // 幅: 4,1,4,1。c=4 → 4+1+4+1+6=16 > 12、c=3 → rows=2, cols=2, 列幅 [4,4] → 10 <= 12
    assert_eq!(format_grid(&e, 12, false), vec!["日本  かな", "a     b"]);
}

#[test]
fn grid_color_does_not_affect_width() {
    let e = vec![
        Entry::new("dir", Kind::Dir),
        Entry::new("f", Kind::File),
        Entry::new("x", Kind::Executable),
    ];
    assert_eq!(
        format_grid(&e, 80, true),
        vec!["\x1b[1;34mdir\x1b[0m  f  \x1b[1;32mx\x1b[0m"]
    );
}

#[test]
fn mode_strings() {
    let mut e = Entry::new("d", Kind::Dir);
    assert_eq!(mode_string(&e), "drwxr-xr-x");
    e.kind = Kind::File;
    e.mode = 0o640;
    assert_eq!(mode_string(&e), "-rw-r-----");
    e.kind = Kind::Symlink;
    e.mode = 0o777;
    assert_eq!(mode_string(&e), "lrwxrwxrwx");
    e.kind = Kind::Executable;
    e.mode = 0o701;
    assert_eq!(mode_string(&e), "-rwx-----x");
}

#[test]
fn long_format_alignment() {
    let mut d = Entry::new("sub", Kind::Dir);
    d.nlink = 12;
    d.size = 64;
    let mut f = Entry::new("a.txt", Kind::File);
    f.size = 12345;
    let mut l = Entry::new("link", Kind::Symlink);
    l.size = 5;
    l.link_target = Some("a.txt".into());
    let lines = format_long(&[d.clone(), f, l], false);
    assert_eq!(
        lines,
        vec![
            "drwxr-xr-x 12    64 sub",
            "-rw-r--r--  1 12345 a.txt",
            "lrwxrwxrwx  1     5 link -> a.txt",
        ]
    );
    assert_eq!(format_long(&[d], true), vec!["drwxr-xr-x 12 64 \x1b[1;34msub\x1b[0m"]);
}

#[test]
fn read_entries_kinds_and_sorting() {
    let dir = fixture("read");
    let entries = read_entries(&dir, false).unwrap();
    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(names, vec!["a.txt", "link", "run.sh", "sub"]);
    let kinds: Vec<Kind> = entries.iter().map(|e| e.kind).collect();
    assert_eq!(kinds, vec![Kind::File, Kind::Symlink, Kind::Executable, Kind::Dir]);
    assert_eq!(entries[0].size, 5);
    assert_eq!(entries[0].mode, 0o644);
    assert_eq!(entries[2].mode, 0o755);
    assert_eq!(entries[1].link_target.as_deref(), Some("a.txt"));
    assert_eq!(entries[0].link_target, None);
}

#[test]
fn read_entries_all_includes_hidden() {
    let dir = fixture("all");
    let names: Vec<String> = read_entries(&dir, true).unwrap().into_iter().map(|e| e.name).collect();
    assert_eq!(names, vec![".hidden", "a.txt", "link", "run.sh", "sub"]);
    assert!(read_entries(&dir.join("nope"), false).is_err());
}

#[test]
fn parse_args_cases() {
    let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>();
    let (o, p) = parse_args(&s(&[])).unwrap();
    assert_eq!(o, Options::default());
    assert_eq!(p, vec!["."]);
    let (o, p) = parse_args(&s(&["-la", "/tmp", "x"])).unwrap();
    assert!(o.all && o.long && !o.color);
    assert_eq!(p, vec!["/tmp", "x"]);
    let (o, _) = parse_args(&s(&["-l", "-a"])).unwrap();
    assert!(o.all && o.long);
    assert_eq!(parse_args(&s(&["-z"])), Err("invalid option: -z".to_string()));
}
