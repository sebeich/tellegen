//! A small Rust lexer: identifiers, punctuation and literals with their byte
//! offsets and lines, and the comments kept apart, so the scanner can match
//! braces and names without ever looking inside a string or a comment.
//!
//! @block Lexer

/// One token of code.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tok {
    /// An identifier or keyword (`r#name` without its prefix).
    Ident(String),
    /// One punctuation character.
    Punct(char),
    /// A string, character or number literal.
    Literal,
    /// A lifetime or loop label.
    Lifetime,
}

/// A token with its position.
#[derive(Clone, Debug)]
pub struct Token {
    pub tok: Tok,
    /// Byte offset of the first character.
    pub start: usize,
    /// Byte offset after the last character.
    pub end: usize,
    /// 1-based line.
    pub line: usize,
}

impl Token {
    pub fn ident(&self) -> Option<&str> {
        match &self.tok {
            Tok::Ident(s) => Some(s),
            _ => None,
        }
    }

    pub fn is(&self, c: char) -> bool {
        self.tok == Tok::Punct(c)
    }
}

/// What a line comment is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommentKind {
    /// `///`: documents the item after it.
    Outer,
    /// `//!`: documents the enclosing module.
    Inner,
    /// `//`: an ordinary comment.
    Plain,
}

/// One line comment, without its marker and the single space after it.
#[derive(Clone, Debug)]
pub struct Comment {
    pub kind: CommentKind,
    pub text: String,
    pub start: usize,
    pub line: usize,
}

/// A lexed file.
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub comments: Vec<Comment>,
}

/// Lex Rust source text. Block comments are skipped (they carry no block
/// tags); unknown characters become punctuation.
#[must_use]
pub fn lex(src: &str) -> Lexed {
    let b = src.as_bytes();
    let mut tokens = Vec::new();
    let mut comments = Vec::new();
    let mut i = 0;
    let mut line = 1;
    let n = b.len();
    // Advance over `from..to`, counting newlines.
    let count = |from: usize, to: usize| b[from..to].iter().filter(|&&c| c == b'\n').count();
    while i < n {
        let c = b[i];
        if c == b'\n' {
            line += 1;
            i += 1;
        } else if c.is_ascii_whitespace() {
            i += 1;
        } else if c == b'/' && b.get(i + 1) == Some(&b'/') {
            let end = src[i..].find('\n').map_or(n, |k| i + k);
            let body = &src[i + 2..end];
            let (kind, text) = if let Some(rest) = body.strip_prefix('!') {
                (CommentKind::Inner, rest)
            } else if body.starts_with('/') && !body.starts_with("//") {
                (CommentKind::Outer, &body[1..])
            } else {
                (CommentKind::Plain, body)
            };
            let text = text.strip_prefix(' ').unwrap_or(text).trim_end();
            comments.push(Comment {
                kind,
                text: text.to_owned(),
                start: i,
                line,
            });
            i = end;
        } else if c == b'/' && b.get(i + 1) == Some(&b'*') {
            // Nested block comment.
            let mut depth = 0usize;
            let mut j = i;
            while j < n {
                if b[j] == b'/' && b.get(j + 1) == Some(&b'*') {
                    depth += 1;
                    j += 2;
                } else if b[j] == b'*' && b.get(j + 1) == Some(&b'/') {
                    depth -= 1;
                    j += 2;
                    if depth == 0 {
                        break;
                    }
                } else {
                    j += 1;
                }
            }
            line += count(i, j.min(n));
            i = j.min(n);
        } else if let Some(end) = raw_string_end(b, i) {
            tokens.push(Token {
                tok: Tok::Literal,
                start: i,
                end,
                line,
            });
            line += count(i, end);
            i = end;
        } else if c == b'"' || (c == b'b' && b.get(i + 1) == Some(&b'"')) {
            let mut j = if c == b'b' { i + 2 } else { i + 1 };
            while j < n && b[j] != b'"' {
                j += if b[j] == b'\\' { 2 } else { 1 };
            }
            let end = (j + 1).min(n);
            tokens.push(Token {
                tok: Tok::Literal,
                start: i,
                end,
                line,
            });
            line += count(i, end);
            i = end;
        } else if c == b'\'' || (c == b'b' && b.get(i + 1) == Some(&b'\'')) {
            let q = if c == b'b' { i + 1 } else { i };
            if let Some(end) = char_literal_end(src, q) {
                tokens.push(Token {
                    tok: Tok::Literal,
                    start: i,
                    end,
                    line,
                });
                i = end;
            } else {
                // A lifetime or label: `'a`, `'static`, `'outer`.
                let mut j = q + 1;
                while j < n && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                    j += 1;
                }
                tokens.push(Token {
                    tok: Tok::Lifetime,
                    start: i,
                    end: j,
                    line,
                });
                i = j;
            }
        } else if c.is_ascii_digit() {
            let mut j = i + 1;
            while j < n
                && (b[j].is_ascii_alphanumeric()
                    || b[j] == b'_'
                    || (b[j] == b'.' && b.get(j + 1).is_some_and(u8::is_ascii_digit)))
            {
                // `1e-5`, `2.0e+3`: a sign after an exponent belongs to the number.
                if (b[j] == b'e' || b[j] == b'E')
                    && matches!(b.get(j + 1), Some(b'+' | b'-'))
                    && b[i..j]
                        .iter()
                        .all(|d| d.is_ascii_digit() || *d == b'.' || *d == b'_')
                {
                    j += 1;
                }
                j += 1;
            }
            tokens.push(Token {
                tok: Tok::Literal,
                start: i,
                end: j,
                line,
            });
            i = j;
        } else if c.is_ascii_alphabetic() || c == b'_' {
            let mut s = i;
            if c == b'r' && b.get(i + 1) == Some(&b'#') {
                s = i + 2;
            }
            let mut j = s;
            while j < n && (b[j].is_ascii_alphanumeric() || b[j] == b'_') {
                j += 1;
            }
            if j == s {
                // `r#` followed by no identifier: treat `r` alone.
                j = i + 1;
                s = i;
            }
            tokens.push(Token {
                tok: Tok::Ident(src[s..j].to_owned()),
                start: i,
                end: j,
                line,
            });
            i = j;
        } else {
            let ch = src[i..].chars().next().unwrap_or('?');
            let w = ch.len_utf8();
            tokens.push(Token {
                tok: Tok::Punct(ch),
                start: i,
                end: i + w,
                line,
            });
            i += w;
        }
    }
    Lexed { tokens, comments }
}

/// The end of a raw string (`r"…"`, `r#"…"#`, `br#"…"#`) starting at `i`.
fn raw_string_end(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    if b.get(j) == Some(&b'b') {
        j += 1;
    }
    if b.get(j) != Some(&b'r') {
        return None;
    }
    // Not part of an identifier like `for` or `bar`.
    if i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_') {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    while j < b.len() {
        if b[j] == b'"' && (0..hashes).all(|k| b.get(j + 1 + k) == Some(&b'#')) {
            return Some(j + 1 + hashes);
        }
        j += 1;
    }
    Some(b.len())
}

/// The end of a character literal whose opening quote is at `q`, or `None`
/// for a lifetime.
fn char_literal_end(src: &str, q: usize) -> Option<usize> {
    let rest = &src[q + 1..];
    let mut chars = rest.char_indices();
    let (_, first) = chars.next()?;
    if first == '\\' {
        // Escape (`'\n'`, `'\''`, `'\u{41}'`): the closing quote after the
        // escaped character.
        let close = rest.get(2..)?.find('\'')? + 2;
        return Some(q + 1 + close + 1);
    }
    let (k, second) = chars.next()?;
    (second == '\'').then_some(q + 1 + k + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn idents(src: &str) -> Vec<String> {
        lex(src)
            .tokens
            .iter()
            .filter_map(|t| t.ident().map(str::to_owned))
            .collect()
    }

    #[test]
    fn strings_and_comments_hide_their_contents() {
        let src = "let a = \"fn hidden {\"; // fn nope\nlet b = r#\"x \" y\"#; /* fn no */ 'c';";
        assert_eq!(idents(src), ["let", "a", "let", "b"]);
        let lexed = lex(src);
        assert_eq!(lexed.comments.len(), 1);
        assert_eq!(lexed.comments[0].text, "fn nope");
    }

    #[test]
    fn lifetimes_are_not_char_literals() {
        let lexed = lex("fn f<'a>(x: &'a str) -> char { 'x' }");
        let lifetimes = lexed
            .tokens
            .iter()
            .filter(|t| t.tok == Tok::Lifetime)
            .count();
        assert_eq!(lifetimes, 2);
        assert!(lexed.tokens.iter().any(|t| t.tok == Tok::Literal));
    }

    #[test]
    fn doc_comment_kinds_and_lines() {
        let lexed = lex("//! inner\n/// outer\n//// plain\nfn x() {}\n");
        let kinds: Vec<_> = lexed.comments.iter().map(|c| (c.kind, c.line)).collect();
        assert_eq!(
            kinds,
            [
                (CommentKind::Inner, 1),
                (CommentKind::Outer, 2),
                (CommentKind::Plain, 3)
            ]
        );
        assert_eq!(lexed.tokens[0].line, 4);
    }

    #[test]
    fn exponents_stay_in_numbers() {
        let lexed = lex("let t = 1e-9 * h; let r = 0..n;");
        assert!(idents("let t = 1e-9 * h;").contains(&"h".to_owned()));
        assert!(lexed.tokens.iter().any(|t| t.ident() == Some("n")));
    }
}
