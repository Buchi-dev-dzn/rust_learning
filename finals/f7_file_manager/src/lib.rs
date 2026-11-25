//! ファイルマネージャの状態と、libc 以外に依存しない小さな TUI フレームワーク。
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect { pub x: u16, pub y: u16, pub width: u16, pub height: u16 }
impl Rect {
    pub const fn new(x: u16, y: u16, width: u16, height: u16) -> Self { Self { x, y, width, height } }
    pub fn split_horizontal(self, left: u16) -> (Self, Self) {
        let left = left.min(self.width);
        (Self::new(self.x, self.y, left, self.height), Self::new(self.x.saturating_add(left), self.y, self.width - left, self.height))
    }
    pub fn inner(self) -> Self {
        Self::new(self.x.saturating_add(1), self.y.saturating_add(1), self.width.saturating_sub(2), self.height.saturating_sub(2))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell { pub ch: char, pub selected: bool }
impl Default for Cell { fn default() -> Self { Self { ch: ' ', selected: false } } }
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer { pub width: u16, pub height: u16, cells: Vec<Cell> }
impl Buffer {
    pub fn new(width: u16, height: u16) -> Self { Self { width, height, cells: vec![Cell::default(); width as usize * height as usize] } }
    pub fn get(&self, x: u16, y: u16) -> Option<Cell> {
        if x >= self.width || y >= self.height { None } else { Some(self.cells[y as usize * self.width as usize + x as usize]) }
    }
    pub fn put(&mut self, x: u16, y: u16, ch: char, selected: bool) {
        if x < self.width && y < self.height && !ch.is_control() {
            self.cells[y as usize * self.width as usize + x as usize] = Cell { ch, selected };
        }
    }
    pub fn text(&mut self, area: Rect, text: &str, selected: bool) {
        // TODO: area と画面の右端でクリップし、全角文字を 2 セルとして書く。
        let _ = (area, text, selected);
        todo!()
    }
    pub fn line(&self, y: u16) -> String {
        if y >= self.height { return String::new(); }
        (0..self.width).map(|x| self.get(x, y).unwrap().ch).collect()
    }
    /// 変更セルのみ移動・色設定して出力。サイズ変更時は全セルを描画。
    pub fn diff(&self, previous: &Self) -> String {
        // TODO: previous と変更セルを比較し ANSI カーソル移動と反転色を出力。
        let _ = previous;
        todo!()
    }
}

pub fn display_width(ch: char) -> usize {
    let n = ch as u32;
    if ch.is_control() || (0x300..=0x36f).contains(&n) { 0 }
    else if (0x1100..=0x115f).contains(&n) || (0x2e80..=0xa4cf).contains(&n)
        || (0xac00..=0xd7a3).contains(&n) || (0xf900..=0xfaff).contains(&n)
        || (0xfe10..=0xfe6f).contains(&n) || (0xff01..=0xff60).contains(&n)
        || (0x1f300..=0x1faff).contains(&n) { 2 } else { 1 }
}

pub trait Widget { fn render(&self, area: Rect, buffer: &mut Buffer); }
pub struct FileList<'a> { pub entries: &'a [Entry], pub selected: usize, pub offset: usize }
impl Widget for FileList<'_> {
    fn render(&self, area: Rect, buffer: &mut Buffer) {
        for (row, entry) in self.entries.iter().enumerate().skip(self.offset).take(area.height as usize) {
            let y = area.y.saturating_add((row - self.offset) as u16);
            buffer.text(Rect::new(area.x, y, area.width, 1),
                &format!("{} {}", if entry.is_dir { "▸" } else { " " }, entry.name), row == self.selected);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry { pub name: String, pub is_dir: bool }
#[derive(Debug)]
pub struct FileManager { pub cwd: PathBuf, pub entries: Vec<Entry>, pub selected: usize, pub offset: usize, pub message: String }
impl FileManager {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let cwd = path.as_ref().canonicalize()?;
        let mut app = Self { cwd, entries: Vec::new(), selected: 0, offset: 0, message: String::new() };
        app.refresh()?;
        Ok(app)
    }
    pub fn refresh(&mut self) -> io::Result<()> {
        let mut entries = Vec::new();
        for result in fs::read_dir(&self.cwd)? {
            let entry = result?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let is_dir = entry.file_type()?.is_dir();
            entries.push(Entry { name, is_dir });
        }
        // TODO: ディレクトリ優先、その後名前順に並べる。
        // entries.sort_by(...);
        let old = self.entries.get(self.selected).map(|e| (&e.name, e.is_dir));
        self.selected = old.and_then(|(name, dir)| entries.iter().position(|e| &e.name == name && e.is_dir == dir))
            .unwrap_or_else(|| self.selected.min(entries.len().saturating_sub(1)));
        self.entries = entries;
        Ok(())
    }
    pub fn down(&mut self) { // TODO: 最後の行では止まる。
        let _ = self;
    }
    pub fn up(&mut self) { // TODO: 最初の行では止まる。
        let _ = self;
    }
    pub fn scroll(&mut self, height: usize) {
        if height == 0 { return; }
        if self.selected < self.offset { self.offset = self.selected; }
        else if self.selected >= self.offset.saturating_add(height) { self.offset = self.selected + 1 - height; }
    }
    pub fn parent(&mut self) -> io::Result<()> {
        let old = self.cwd.clone();
        if let Some(parent) = old.parent() {
            if parent != old {
                let mut next = Self::open(parent)?;
                if let Some(name) = old.file_name() {
                    if let Some(index) = next.entries.iter().position(|e| std::ffi::OsStr::new(&e.name) == name) { next.selected = index; }
                }
                *self = next;
            }
        }
        Ok(())
    }
    pub fn enter(&mut self) -> io::Result<()> {
        // TODO: ディレクトリ遷移とファイル選択のメッセージ。
        if let Some(entry) = self.entries.get(self.selected) {
            if entry.is_dir {
                let next = Self::open(self.cwd.join(&entry.name))?;
                *self = next;
            } else { self.message = format!("ファイル: {}", entry.name); }
        }
        Ok(())
    }
    pub fn render(&self, buffer: &mut Buffer) {
        let screen = Rect::new(0, 0, buffer.width, buffer.height);
        if screen.height == 0 { return; }
        buffer.text(Rect::new(0, 0, screen.width, 1), &format!(" FILES  {}", self.cwd.display()), true);
        let body = Rect::new(0, 1, screen.width, screen.height.saturating_sub(2));
        let (list, detail) = body.split_horizontal(if body.width >= 40 { body.width * 2 / 3 } else { body.width });
        FileList { entries: &self.entries, selected: self.selected, offset: self.offset }.render(list, buffer);
        if detail.width > 0 {
            let detail = detail.inner();
            if let Some(e) = self.entries.get(self.selected) {
                buffer.text(Rect::new(detail.x, detail.y, detail.width, 1), &e.name, false);
                buffer.text(Rect::new(detail.x, detail.y.saturating_add(1), detail.width, 1),
                    if e.is_dir { "ディレクトリ" } else { "ファイル" }, false);
            }
        }
        buffer.text(Rect::new(0, screen.height - 1, screen.width, 1),
            &format!(" j/k:移動  Enter:開く  h:親  r:更新  q:終了   {}", self.message), true);
    }
}
