use super::src::{SrcLoc, SrcRange};
use crate::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum Lexeme {
    Ident(String, usize),
    Str(String),
    Num(String),
    Def,
    Let,
    True,
    False,
    Null,
    Pipe,
    Dollar,
    At,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Token(pub Lexeme, pub SrcRange);

pub fn lex(code: &str) -> Result<Vec<Token>, Vec<Error>> {
    enum State {
        Default,
        String(String, bool),
        Ident(String, usize),
        Sym(String, usize),
        Num(String),
    }

    fn is_singular(c: char) -> bool {
        match c {
            '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' => true,
            _ => false,
        }
    }

    let mut state = State::Default;
    let mut chars = code.chars();
    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    let mut sloc = SrcLoc::new(1, 1);
    let mut cloc = sloc;
    let mut range_len = 0;

    loop {
        let mut incr = true;
        let c = chars.clone().next().unwrap_or('\0');

        if let State::Default = state {
            // Reset lexeme location
            sloc = cloc;
            range_len = 0;
        }
        let range = SrcRange::new(sloc, range_len);

        match &mut state {
            State::Default => match c {
                '"' /*"*/ => state = State::String(String::from("\""), false),
                '|' => tokens.push(Token(Lexeme::Pipe, range.grow_by(1))),
                '$' => tokens.push(Token(Lexeme::Dollar, range.grow_by(1))),
                '@' => tokens.push(Token(Lexeme::At, range.grow_by(1))),
                c if is_singular(c) => tokens.push(Token(Lexeme::Ident(c.to_string(), 0), range.grow_by(1))),
                c if c.is_whitespace() => {},
                c if c.is_alphabetic() || c == '_' => state = State::Ident(c.to_string(), 0),
                c if c.is_numeric() => state = State::Num(c.to_string()),
                c if c.is_ascii_punctuation() => state = State::Sym(c.to_string(), 0),
                '\0' => break,
                c => errors.push(Error::unexpected_char(c).at(range)),
            },
            State::String(text, escaped) => match c {
                '"' /*"*/ => {
                    tokens.push(Token(Lexeme::Str(text.clone()), range));
                    state = State::Default;
                },
                '\0' => break,
                c => text.push(c),
            },
            State::Ident(text, arity) => match c {
                c if (c.is_alphanumeric() || c == '_') && *arity == 0 => text.push(c),
                '@' => *arity += 1,
                c => {
                    tokens.push(Token(
                        match text.as_str() {
                            "def" => Lexeme::Def,
                            "let" => Lexeme::Let,
                            "true" => Lexeme::True,
                            "false" => Lexeme::False,
                            "null" => Lexeme::Null,
                            _ => Lexeme::Ident(text.clone(), *arity),
                        },
                        range,
                    ));
                    state = State::Default;
                    incr = false;
                },
            },
            State::Num(text) => match c {
                c if c.is_alphanumeric() => text.push(c),
                c => {
                    tokens.push(Token(Lexeme::Num(text.clone()), range));
                    state = State::Default;
                    incr = false;
                },
            },
            State::Sym(text, arity) => match c {
                '@' => *arity += 1,
                c if c.is_ascii_punctuation() && *arity == 0 => text.push(c),
                c => {
                    tokens.push(Token(Lexeme::Ident(text.clone(), *arity), range));
                    state = State::Default;
                    incr = false;
                },
            },
        }

        if incr {
            match chars.next() {
                Some('\n') => {
                    cloc.line += 1;
                    cloc.col = 1;
                },
                _ => cloc.col += 1,
            }
            range_len += 1;
        }
    }

    match state {
        State::Default => {},
        State::String(_, _) => errors.push(Error::expected_delimiter('"' /*"*/).at(SrcRange::new(sloc, range_len))),
        _ => panic!(),
    }

    if errors.len() > 0 {
        Err(errors)
    } else {
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_idents() {
        assert_eq!(
            lex("test"),
            Ok(vec![
                Token(Lexeme::Ident("test".to_string(), 0), SrcRange::new(SrcLoc::new(1, 1), 4)),
            ]),
        );

        assert_eq!(
            lex("foobar@@@"),
            Ok(vec![
                Token(Lexeme::Ident("foobar".to_string(), 3), SrcRange::new(SrcLoc::new(1, 1), 9)),
            ]),
        );

        assert_eq!(
            lex("test f00@@@bleugh"),
            Ok(vec![
                Token(Lexeme::Ident("test".to_string(), 0), SrcRange::new(SrcLoc::new(1, 1), 4)),
                Token(Lexeme::Ident("f00".to_string(), 3), SrcRange::new(SrcLoc::new(1, 6), 6)),
                Token(Lexeme::Ident("bleugh".to_string(), 0), SrcRange::new(SrcLoc::new(1, 12), 6)),
            ]),
        );
    }

    #[test]
    fn lex_singular() {
        assert_eq!(
            lex("[,];|(){}["),
            Ok(vec![
                Token(Lexeme::Ident("[".to_string(), 0), SrcRange::new(SrcLoc::new(1, 1), 1)),
                Token(Lexeme::Ident(",".to_string(), 0), SrcRange::new(SrcLoc::new(1, 2), 1)),
                Token(Lexeme::Ident("]".to_string(), 0), SrcRange::new(SrcLoc::new(1, 3), 1)),
                Token(Lexeme::Ident(";".to_string(), 0), SrcRange::new(SrcLoc::new(1, 4), 1)),
                Token(Lexeme::Pipe, SrcRange::new(SrcLoc::new(1, 5), 1)),
                Token(Lexeme::Ident("(".to_string(), 0), SrcRange::new(SrcLoc::new(1, 6), 1)),
                Token(Lexeme::Ident(")".to_string(), 0), SrcRange::new(SrcLoc::new(1, 7), 1)),
                Token(Lexeme::Ident("{".to_string(), 0), SrcRange::new(SrcLoc::new(1, 8), 1)),
                Token(Lexeme::Ident("}".to_string(), 0), SrcRange::new(SrcLoc::new(1, 9), 1)),
                Token(Lexeme::Ident("[".to_string(), 0), SrcRange::new(SrcLoc::new(1, 10), 1)),
            ]),
        );
    }

    #[test]
    fn lex_eof() {
        assert_eq!(
            lex("\"testing this!"),
            Err(vec![Error::expected_delimiter('"' /*"*/).at(SrcRange::new(SrcLoc::new(1, 1), 14))]),
        );
    }
}