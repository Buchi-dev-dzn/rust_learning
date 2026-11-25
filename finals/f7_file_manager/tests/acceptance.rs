//! F7 受け入れテスト。端末を起動せず、自作フレームワークと状態を検証する。
use f7_file_manager::{Buffer, Entry, FileList, FileManager, Rect, Widget, display_width};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn geometry_clamps_and_splits() {
    let r = Rect::new(3, 4, 9, 5);
    assert_eq!(r.split_horizontal(4), (Rect::new(3, 4, 4, 5), Rect::new(7, 4, 5, 5)));
    assert_eq!(r.split_horizontal(99).1.width, 0);
    assert_eq!(Rect::new(0, 0, 1, 1).inner().width, 0);
}
#[test]
fn text_clips_unicode_and_out_of_bounds() {
    let mut b = Buffer::new(6, 2);
    b.text(Rect::new(1, 0, 4, 1), "aあbZ", true);
    assert_eq!(b.line(0), " aあ b ");
    assert!(b.get(2, 0).unwrap().selected);
    b.text(Rect::new(5, 0, 1, 1), "あ", false);
    assert_eq!(b.line(0), " aあ b ");
    b.text(Rect::new(0, 9, 3, 1), "x", false);
    assert_eq!(display_width('あ'), 2);
    assert_eq!(display_width('a'), 1);
}
#[test]
fn diff_changes_only_and_restores_style() {
    let prev = Buffer::new(4, 1);
    let mut next = Buffer::new(4, 1);
    next.put(1, 0, 'X', true);
    let ansi = next.diff(&prev);
    assert!(ansi.contains("\x1b[1;2H\x1b[7mX"));
    assert!(!ansi.contains("\x1b[1;1H"));
    assert!(ansi.ends_with("\x1b[0m"));
    assert_eq!(next.diff(&next), "\x1b[0m");
    assert!(next.diff(&Buffer::new(2, 1)).contains("\x1b[2J"));
}
#[test]
fn list_widget_selection_and_offset() {
    let entries = (0..5).map(|i| Entry { name: format!("item{i}"), is_dir: i == 2 }).collect::<Vec<_>>();
    let mut b = Buffer::new(12, 2);
    FileList { entries: &entries, selected: 2, offset: 2 }.render(Rect::new(0, 0, 12, 2), &mut b);
    assert!(b.line(0).contains("▸ item2"));
    assert!(b.get(0, 0).unwrap().selected);
    assert!(b.line(1).contains("item3"));
    assert!(!b.get(0, 1).unwrap().selected);
}
struct Fixture(std::path::PathBuf);
impl Fixture {
    fn new() -> Self {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("f7-{}-{nonce}", std::process::id()));
        fs::create_dir(&dir).unwrap();
        Self(dir)
    }
}
impl Drop for Fixture { fn drop(&mut self) { fs::remove_dir_all(&self.0).unwrap(); } }
#[test]
fn filesystem_sort_navigation_refresh_and_clamp() {
    let f = Fixture::new();
    fs::create_dir(f.0.join("z_dir")).unwrap();
    fs::create_dir(f.0.join("a_dir")).unwrap();
    fs::write(f.0.join("a_file"), "hello").unwrap();
    let mut app = FileManager::open(&f.0).unwrap();
    assert_eq!(app.entries.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), ["a_dir", "z_dir", "a_file"]);
    app.down();
    app.down();
    app.down();
    assert_eq!(app.selected, 2);
    app.scroll(1);
    assert_eq!(app.offset, 2);
    app.up();
    app.scroll(1);
    assert_eq!(app.offset, 1);
    fs::write(f.0.join("b_file"), "world").unwrap();
    app.refresh().unwrap();
    assert_eq!(app.entries[app.selected].name, "z_dir");
    app.enter().unwrap();
    assert_eq!(app.cwd, f.0.join("z_dir").canonicalize().unwrap());
    assert!(app.entries.is_empty());
    app.parent().unwrap();
    assert_eq!(app.entries[app.selected].name, "z_dir");
    app.down();
    app.enter().unwrap();
    assert_eq!(app.message, "ファイル: a_file");
}
#[test]
fn empty_directory_and_tiny_screen() {
    let f = Fixture::new();
    let mut app = FileManager::open(&f.0).unwrap();
    app.down();
    app.up();
    app.enter().unwrap();
    assert_eq!(app.selected, 0);
    let mut b = Buffer::new(0, 0);
    app.render(&mut b);
    let mut b = Buffer::new(3, 1);
    app.render(&mut b);
    assert_eq!(b.width, 3);
}
