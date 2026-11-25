//! # コピーしないトークナイザ
//!
//! シェルやエディタのコマンドライン (`:open "my file.txt" | wc`) を、
//! 元の文字列のスライスを返すトークンに分解しよう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! 規則:
//! - 空白 (`char::is_whitespace`) はトークンの区切りで、トークンにはならない
//! - `|` は 1 文字で `Token::Pipe`
//! - `"` で始まると次の `"` までが `Token::Quoted(中身)` (引用符は含めない)。
//!   閉じ `"` がなければ末尾までを中身とする
//! - それ以外は、空白・`|`・`"` の手前までが `Token::Word`
//!
//! - 関連: 第10章「構造体に参照を持たせる」「Iterator を実装する」

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Token<'a> {
    Word(&'a str),
    Quoted(&'a str),
    Pipe,
}

/// 入力の「まだ読んでいない残り」を持つ
pub struct Tokenizer<'a> {
    rest: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Tokenizer { rest: input }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    // トークンは Tokenizer ではなく元の入力 ('a) から借りる
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Token<'a>> {
        // TODO:
        // 1. self.rest の先頭の空白を飛ばす (trim_start)
        // 2. 空なら None
        // 3. 先頭の文字で分岐し、トークンを作り、self.rest を「残り」に更新する
        //    ヒント: '|' と '"' は 1 バイトなので &s[1..] で次へ進める
        todo!()
    }
}

/// 便利関数: 全トークンを Vec にする
pub fn tokenize(input: &str) -> Vec<Token<'_>> {
    Tokenizer::new(input).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use Token::*;

    #[test]
    fn words() {
        assert_eq!(tokenize("ls -la  /tmp"), vec![Word("ls"), Word("-la"), Word("/tmp")]);
    }

    #[test]
    fn empty_and_blank() {
        assert_eq!(tokenize(""), vec![]);
        assert_eq!(tokenize("   \t "), vec![]);
    }

    #[test]
    fn pipes_without_spaces() {
        assert_eq!(
            tokenize("cat a|grep x | wc"),
            vec![Word("cat"), Word("a"), Pipe, Word("grep"), Word("x"), Pipe, Word("wc")]
        );
    }

    #[test]
    fn quoted() {
        assert_eq!(
            tokenize(r#":open "my file.txt" now"#),
            vec![Word(":open"), Quoted("my file.txt"), Word("now")]
        );
        assert_eq!(tokenize(r#"echo"hi""#), vec![Word("echo"), Quoted("hi")]);
        assert_eq!(tokenize(r#""""#), vec![Quoted("")]);
    }

    #[test]
    fn unterminated_quote() {
        assert_eq!(tokenize(r#"say "hello wor"#), vec![Word("say"), Quoted("hello wor")]);
    }

    #[test]
    fn japanese() {
        assert_eq!(tokenize("検索 \"東京 都\"|件数"), vec![Word("検索"), Quoted("東京 都"), Pipe, Word("件数")]);
    }

    #[test]
    fn tokens_outlive_tokenizer() {
        let line = String::from("a b");
        let first;
        {
            let mut t = Tokenizer::new(&line);
            first = t.next();
        } // Tokenizer はここで破棄されるが、トークンは line から借りているので使える
        assert_eq!(first, Some(Word("a")));
    }
}
