//! Markdown → HTML。コードブロックに「▶ 実行」ボタン用の属性を付け、h2 から目次を作る。

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd, html};

pub struct Rendered {
    pub html: String,
    /// (id, 見出しテキスト)
    pub toc: Vec<(String, String)>,
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

pub fn render(md: &str) -> Rendered {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(md, opts);

    let mut events: Vec<Event> = Vec::new();
    let mut toc = Vec::new();
    // コードブロック収集中: (言語情報, 本文)
    let mut code: Option<(String, String)> = None;
    // 見出し収集中: (レベル, テキスト, 内部イベント)
    let mut heading: Option<(HeadingLevel, String, Vec<Event>)> = None;

    for ev in parser {
        if let Some((_, buf)) = code.as_mut() {
            match ev {
                Event::Text(t) => buf.push_str(&t),
                Event::End(TagEnd::CodeBlock) => {
                    let (info, body) = code.take().unwrap();
                    events.push(Event::Html(code_block_html(&info, &body).into()));
                }
                _ => {}
            }
            continue;
        }
        if let Some((_, text, inner)) = heading.as_mut() {
            match ev {
                Event::End(TagEnd::Heading(level)) => {
                    let (_, text, inner) = heading.take().unwrap();
                    let id = format!("sec-{}", toc.len() + 1);
                    let tag = match level {
                        HeadingLevel::H2 => "h2",
                        HeadingLevel::H3 => "h3",
                        HeadingLevel::H4 => "h4",
                        HeadingLevel::H5 => "h5",
                        HeadingLevel::H6 => "h6",
                        HeadingLevel::H1 => "h1",
                    };
                    let mut inner_html = String::new();
                    html::push_html(&mut inner_html, inner.into_iter());
                    if level == HeadingLevel::H2 {
                        toc.push((id.clone(), text));
                        events.push(Event::Html(
                            format!("<{tag} id=\"{id}\">{inner_html}</{tag}>\n").into(),
                        ));
                    } else {
                        events.push(Event::Html(format!("<{tag}>{inner_html}</{tag}>\n").into()));
                    }
                }
                other => {
                    if let Event::Text(t) | Event::Code(t) = &other {
                        text.push_str(t);
                    }
                    inner.push(other);
                }
            }
            continue;
        }
        match ev {
            Event::Start(Tag::CodeBlock(kind)) => {
                let info = match kind {
                    CodeBlockKind::Fenced(i) => i.to_string(),
                    CodeBlockKind::Indented => String::new(),
                };
                code = Some((info, String::new()));
            }
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some((level, String::new(), Vec::new()));
            }
            other => events.push(other),
        }
    }

    let mut out = String::new();
    html::push_html(&mut out, events.into_iter());
    Rendered { html: out, toc }
}

/// コードが単独で実行可能か (fn main を含み、noplayground 指定がない rust)
pub fn is_runnable(info: &str, body: &str) -> bool {
    let mut parts = info.split(',').map(str::trim);
    let lang = parts.next().unwrap_or("");
    let flags: Vec<&str> = parts.collect();
    lang == "rust"
        && !flags.iter().any(|f| matches!(*f, "noplayground" | "ignore" | "no_run" | "compile_fail"))
        && body.contains("fn main")
}

fn code_block_html(info: &str, body: &str) -> String {
    let lang = info.split(',').next().unwrap_or("").trim();
    let runnable = is_runnable(info, body);
    let lang_label = if lang.is_empty() { "text" } else { lang };
    let mut s = String::new();
    s.push_str(&format!(
        "<div class=\"codeblock\" data-lang=\"{}\"{}>",
        escape(lang_label),
        if runnable { " data-runnable=\"1\"" } else { "" }
    ));
    s.push_str("<div class=\"codeblock-bar\"><span class=\"lang\">");
    s.push_str(&escape(lang_label));
    s.push_str("</span><span class=\"spacer\"></span><button class=\"btn tiny copy\" type=\"button\">コピー</button>");
    if runnable {
        s.push_str("<button class=\"btn tiny primary run\" type=\"button\">▶ 実行</button>");
    }
    s.push_str("</div><pre><code>");
    s.push_str(&escape(body));
    s.push_str("</code></pre>");
    if runnable {
        s.push_str("<pre class=\"run-output\" hidden></pre>");
    }
    s.push_str("</div>\n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runnable_detection() {
        assert!(is_runnable("rust", "fn main() {}"));
        assert!(!is_runnable("rust,noplayground", "fn main() {}"));
        assert!(!is_runnable("rust", "fn foo() {}"));
        assert!(!is_runnable("toml", "fn main"));
    }

    #[test]
    fn toc_and_code() {
        let r = render("## はじめに\ntext\n```rust\nfn main() { println!(\"<hi>\"); }\n```\n### sub\n");
        assert_eq!(r.toc, vec![("sec-1".to_string(), "はじめに".to_string())]);
        assert!(r.html.contains("data-runnable"));
        assert!(r.html.contains("&lt;hi&gt;"));
        assert!(r.html.contains("<h3>sub</h3>"));
    }
}
