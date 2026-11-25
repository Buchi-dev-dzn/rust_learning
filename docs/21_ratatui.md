# 第21章 ratatui: Layout・Widget・StatefulWidget

## この章のゴール

- ratatui で端末の矩形を分割し、複数ペインを描画する
- `Paragraph`、`Block`、`List` を用途に応じて使い分ける
- `ListState` をアプリの状態と同期させ、選択行を表示する
- イベント処理と描画を分離し、端末なしで更新ロジックをテストする

第20章では Model / Message / update / view を分けました。この章では view の描画先を文字列から
ratatui の `Frame` に変えます。入力は引き続き crossterm から受け取ります。
画面が大きくなっても、状態を変更する場所は update のままです。

> 以下の ratatui / crossterm の例は外部クレートを使うため `rust,noplayground` です。
> Cargo プロジェクト p06 で実際の端末上で試してください。演習は std のみで座標と選択状態を練習します。

## Cargo と初期化

ワークスペースには ratatui 0.30 と crossterm 0.29 が指定されています。プロジェクトでは
`ratatui = { workspace = true }`、`crossterm = { workspace = true }` を依存に加えます。
単体のファイルを `rustc` でコンパイルしても外部クレートは見つかりません。

ratatui の `DefaultTerminal` は内部で crossterm を使えます。画面の開始・終了は必ず対にします。

```rust,noplayground
use std::io;
use ratatui::{DefaultTerminal, Frame};
use ratatui::widgets::Paragraph;

fn draw(frame: &mut Frame) {
    frame.render_widget(Paragraph::new("Hello TUI"), frame.area());
}

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    terminal.draw(draw)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore(); // エラーで run が終了しても復元する
    result
}
```

パニック時にも確実に復元する設計は第19章の Drop ガードと同じです。自分で raw モードを
開始した場合、復元の所有者も明確にしてください。二重の初期化は避けます。

## Frame は 1 回の描画にだけ存在する

`terminal.draw(|frame| { ... })` は新しいフレームを作り、クロージャの処理が終わると
前のフレームとの差分を端末に反映します。`Frame` をアプリのフィールドに保存しないでください。
画面は保存された状態から毎回作り直します。リサイズしたときも同じ view を呼べばよいのです。

```rust,noplayground
use ratatui::{Frame, layout::Rect, widgets::Paragraph};

struct App { title: String }

fn view(app: &App, frame: &mut Frame) {
    let area: Rect = frame.area();
    frame.render_widget(Paragraph::new(app.title.as_str()), area);
}
```

`frame.area()` はそのフレームの端末全体です。矩形の `x` と `y` は左上を起点とする
0 始まり、`width` と `height` はセル単位です。右端は `x + width` の手前です。
小さな端末では幅・高さが 0 の矩形もあり得ます。

## Layout: 領域を先に決める

タイトル行・本文・フッターを縦に並べるとき、手で y を足し算するより `Layout` を使います。
`Constraint::Length(1)` は固定の 1 行、`Constraint::Min(0)` は残った部分を取ります。

```rust,noplayground
use ratatui::{Frame, layout::{Constraint, Direction, Layout}, widgets::Paragraph};

fn draw(frame: &mut Frame) {
    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(frame.area());
    frame.render_widget(Paragraph::new("Files"), rows[0]);
    frame.render_widget(Paragraph::new("main pane"), rows[1]);
    frame.render_widget(Paragraph::new("q: quit"), rows[2]);
}
```

`Length`、`Min`、`Max`、`Percentage`、`Ratio`、`Fill` は要件に合わせて選びます。
制約は「ちょうどこの矩形を保証する」命令ではなく、狭い端末で調整される割当て条件です。
`rows[1]` も 0 行になり得るので、ウィジェット側も空の領域を許容する設計にします。

## 二段階の分割と内側の余白

縦に分けた本文を左右に分けます。外側の `Block` は枠を描き、内容には `inner` を渡します。
枠と内容に同じ領域を渡すと文字が枠の上に描かれるので注意します。

```rust,noplayground
use ratatui::{Frame, layout::{Constraint, Direction, Layout}, widgets::{Block, Borders, Paragraph}};

fn draw(frame: &mut Frame) {
    let columns = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(frame.area());
    let left = Block::default().title("Files").borders(Borders::ALL);
    let inner = left.inner(columns[0]);
    frame.render_widget(left, columns[0]);
    frame.render_widget(Paragraph::new("README.md\nsrc/"), inner);
    frame.render_widget(
        Paragraph::new("Preview").block(Block::default().borders(Borders::ALL)),
        columns[1],
    );
}
```

`Block::inner` の幅や高さは 0 になり得ます。短いテキストなら widget のクリッピングに任せられますが、
独自の位置計算をする場合は端末寸法の境界条件をテストします。
演習 ex01 はこの座標計算を外部クレートなしで再現します。

## Paragraph: テキストと装飾

`Paragraph` は複数行のテキスト、`Line` は一行、`Span` は一部分を表します。
状態を表示するだけなら、入力イベントで文字列を組み立てる必要はありません。

```rust,noplayground
use ratatui::{Frame, style::{Color, Modifier, Style}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Wrap}};

fn draw(frame: &mut Frame) {
    let line = Line::from(vec![
        Span::styled("Status: ", Style::default().add_modifier(Modifier::BOLD)),
        Span::styled("Ready", Style::default().fg(Color::Green)),
    ]);
    let paragraph = Paragraph::new(line)
        .block(Block::default().title("Info").borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(paragraph, frame.area());
}
```

Rust の `String::len()` はバイト数です。多バイト文字の表示幅は第16章で学んだ通り異なります。
ratatui のウィジェットは端末セルを扱いますが、自作の検索カーソルや独自整形では表示幅を
別途計算してください。結合文字や絵文字を `chars().count()` だけで配置しないこと。

## List: 項目の描画

`List` は項目列を表示します。選択行を持たない一覧なら通常の `render_widget` で十分です。
選択を伴う場合は状態を渡す `render_stateful_widget` を使います。

```rust,noplayground
use ratatui::{Frame, style::{Color, Style}, widgets::{Block, Borders, List, ListItem, ListState}};

struct App { names: Vec<String>, selected: ListState }

fn view(app: &mut App, frame: &mut Frame) {
    let items: Vec<ListItem> = app.names.iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();
    let list = List::new(items)
        .block(Block::default().title("Files").borders(Borders::ALL))
        .highlight_style(Style::default().bg(Color::Blue))
        .highlight_symbol("> ");
    frame.render_stateful_widget(list, frame.area(), &mut app.selected);
}
```

`ListState::select(Some(i))` が選択行、`select(None)` が未選択です。選択 index は項目数を
超えないように更新します。描画処理の中でキー判定を行わず、update で状態を変えます。

## update と view の責任分担

```rust
#[derive(Default)]
struct App { items: Vec<&'static str>, selected: Option<usize> }

enum Msg { Next, Previous, Clear }

fn update(app: &mut App, msg: Msg) {
    let len = app.items.len();
    app.selected = match (msg, app.selected) {
        (Msg::Clear, _) => None,
        _ if len == 0 => None,
        (Msg::Next, Some(i)) => Some((i + 1) % len),
        (Msg::Next, None) => Some(0),
        (Msg::Previous, Some(0) | None) => Some(len - 1),
        (Msg::Previous, Some(i)) => Some(i - 1),
    };
}

fn main() {
    let mut app = App { items: vec!["one", "two"], ..App::default() };
    update(&mut app, Msg::Next);
    update(&mut app, Msg::Previous);
    println!("selected: {:?}", app.selected);
}
```

この標準ライブラリだけの例の `selected` は Model 側の値です。実アプリでは `ListState` に
直接状態を保持する方法と、Model の `Option<usize>` を描画時に移す方法があります。
所有者を一つに決め、二つの値がずれないようにします。演習 ex02 では項目の置換・空リストも扱います。

## イベントループと後片付け

描画 → イベント待ち → 更新を繰り返します。`KeyEventKind::Press` のみを受け付けると
端末によるキー解放イベントの二重処理を防げます。リサイズ時には再描画すればよいです。

```rust,noplayground
use std::io;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{DefaultTerminal, widgets::Paragraph};

fn run(mut terminal: DefaultTerminal) -> io::Result<()> {
    let mut count = 0;
    loop {
        terminal.draw(|frame| {
            frame.render_widget(Paragraph::new(format!("count: {count} (q: quit)")), frame.area());
        })?;
        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press { continue; }
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Char('+') => count += 1,
                _ => {}
            }
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let terminal = ratatui::init();
    let result = run(terminal);
    ratatui::restore();
    result
}
```

この例は説明用の最小形です。実際には入力を `Msg` に変換して `update` を呼び、
保存・ファイル読み込みなどの副作用を明示的に分離します。
エラー時だけでなくパニック時の復元経路も設計してください (第19章)。

## よくあるコンパイルエラーと読み方

- `use of unresolved module or unlinked crate ratatui`: `rustc` 単体では依存を解決できません。
  Cargo の依存定義を確認し、外部クレート例を実行ボタンで実行しないでください。
- `cannot borrow ... as mutable`: `render_stateful_widget` は状態に `&mut ListState` を要求します。
  `view` が状態を受ける借用の範囲を短くし、イベント更新と重ねないでください。
- `the trait Widget is not implemented`: `render_widget` に渡す型と `render_stateful_widget` に
  渡す型を混同していないか確認します。`List` は両方の描画経路があります。
- `index out of bounds`: `Layout::split` の結果の個数は constraints と一致させます。
  リストの index は項目数変更時に補正します。

## まとめ

ratatui は端末の初期化とフレーム差分を引き受けます。しかしどの矩形に何を置くか、
選択状態をいつ変更するかはアプリの責任です。Layout で領域を分け、Widget で描き、
StatefulWidget に明示的な状態を渡しましょう。純粋な update を残せば端末なしでテストできます。

## 次へ

第22章では `Frame` の裏側にあるセルバッファ、レイアウト、差分描画を自分で設計します。
第21章で使った便利な API がなぜ必要だったのかを、実装から確かめます。
