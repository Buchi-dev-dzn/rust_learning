//! # Widget トレイトとトレイトオブジェクト
//!
//! TUI フレームワークの心臓部「Widget トレイト」を作ろう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `Widget` トレイト: `fn render(&self, area: Rect, buf: &mut Buffer)`
//! - `Label`: テキストを area の左上から 1 行描く (area の幅を超えた分は切り捨て)
//! - `Gauge`: area の 1 行目を `ratio` に応じて `#` と `.` で塗る (塗る数は `(width as f64 * ratio).round()`)
//! - `VStack`: 子ウィジェット `Vec<Box<dyn Widget>>` を 1 行ずつ縦に並べて描く
//! - 関連: 第9章「トレイトオブジェクト dyn Trait」

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// 文字のセルバッファ (第22章で本格的なものを作る)
pub struct Buffer {
    pub width: u16,
    pub height: u16,
    cells: Vec<char>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        Buffer { width, height, cells: vec![' '; width as usize * height as usize] }
    }

    /// 範囲外への書き込みは無視する
    pub fn set(&mut self, x: u16, y: u16, c: char) {
        if x < self.width && y < self.height {
            let i = y as usize * self.width as usize + x as usize;
            self.cells[i] = c;
        }
    }

    /// 1 行ずつ文字列にする
    pub fn lines(&self) -> Vec<String> {
        self.cells
            .chunks(self.width as usize)
            .map(|row| row.iter().collect())
            .collect()
    }
}

// TODO: render メソッドのシグネチャを宣言する
//   fn render(&self, area: Rect, buf: &mut Buffer);
pub trait Widget {
}

pub struct Label {
    pub text: String,
}

impl Label {
    pub fn new(text: &str) -> Self {
        Label { text: text.to_string() }
    }
}

impl Widget for Label {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // TODO: self.text を (area.x, area.y) から 1 文字ずつ buf.set する
        //       area.width を超える分は描かない。area.height が 0 なら何もしない
        todo!()
    }
}

pub struct Gauge {
    pub ratio: f64,
}

impl Widget for Gauge {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // TODO: 幅 area.width のうち先頭 (width * ratio).round() 個を '#'、残りを '.' で描く
        todo!()
    }
}

/// 子を縦に 1 行ずつ並べるコンテナ。違う型の子を持てるのが dyn の力。
pub struct VStack {
    pub children: Vec<Box<dyn Widget>>,
}

impl Widget for VStack {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        // TODO: i 番目の子を、高さ 1 の Rect { x: area.x, y: area.y + i, width: area.width, height: 1 }
        //       に描く。area.height を超える子は描かない
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full(w: u16, h: u16) -> Rect {
        Rect { x: 0, y: 0, width: w, height: h }
    }

    #[test]
    fn label_renders_and_truncates() {
        let mut buf = Buffer::new(5, 1);
        Label::new("Hello, world").render(full(5, 1), &mut buf);
        assert_eq!(buf.lines(), vec!["Hello"]);
    }

    #[test]
    fn label_respects_offset() {
        let mut buf = Buffer::new(6, 2);
        Label::new("ab").render(Rect { x: 3, y: 1, width: 3, height: 1 }, &mut buf);
        assert_eq!(buf.lines(), vec!["      ", "   ab "]);
    }

    #[test]
    fn gauge_fills_by_ratio() {
        let mut buf = Buffer::new(10, 1);
        Gauge { ratio: 0.3 }.render(full(10, 1), &mut buf);
        assert_eq!(buf.lines(), vec!["###......."]);

        let mut buf = Buffer::new(4, 1);
        Gauge { ratio: 1.0 }.render(full(4, 1), &mut buf);
        assert_eq!(buf.lines(), vec!["####"]);
    }

    #[test]
    fn vstack_with_mixed_children() {
        let ui = VStack {
            children: vec![
                Box::new(Label::new("CPU")),
                Box::new(Gauge { ratio: 0.5 }),
                Box::new(Label::new("q:quit")),
            ],
        };
        let mut buf = Buffer::new(6, 3);
        ui.render(full(6, 3), &mut buf);
        assert_eq!(buf.lines(), vec!["CPU   ", "###...", "q:quit"]);
    }

    #[test]
    fn vstack_clips_to_height() {
        let ui = VStack {
            children: vec![
                Box::new(Label::new("a")),
                Box::new(Label::new("b")),
                Box::new(Label::new("c")),
            ],
        };
        let mut buf = Buffer::new(1, 3);
        ui.render(full(1, 2), &mut buf);
        assert_eq!(buf.lines(), vec!["a", "b", " "]);
    }

    #[test]
    fn widgets_in_a_vec() {
        let widgets: Vec<Box<dyn Widget>> =
            vec![Box::new(Gauge { ratio: 0.0 }), Box::new(Label::new("x"))];
        let mut buf = Buffer::new(3, 1);
        for w in &widgets {
            w.render(full(3, 1), &mut buf);
        }
        assert_eq!(buf.lines(), vec!["x.."]);
    }
}
