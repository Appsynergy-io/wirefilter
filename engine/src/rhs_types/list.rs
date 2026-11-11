use crate::lex::{Lex, LexErrorKind, LexResult, expect};
use serde::Serialize;
use std::str;

#[derive(PartialEq, Eq, Clone, Serialize, Hash, Debug)]
pub struct ListName(Box<str>);

impl From<String> for ListName {
    fn from(src: String) -> Self {
        ListName(src.into_boxed_str())
    }
}

impl Lex<'_> for ListName {
    fn lex(input: &str) -> LexResult<'_, Self> {
        Self::lex_with_hyphens(input, false)
    }
}

impl ListName {
    /// Lexes a list name, taking `-` as a name character when `hyphens` is set.
    pub(crate) fn lex_with_hyphens(input: &str, hyphens: bool) -> LexResult<'_, Self> {
        let mut res = String::new();
        let mut rest;
        let input = expect(input, "$")?;
        let mut iter = input.chars();
        loop {
            rest = iter.as_str();
            match iter.next() {
                Some(c) => match c {
                    'a'..='z' | '0'..='9' | '_' | '.' => res.push(c),
                    '-' if hyphens => res.push(c),
                    _ => {
                        if res.is_empty() {
                            return Err((
                                LexErrorKind::InvalidListName {
                                    name: c.to_string().into_boxed_str(),
                                },
                                input,
                            ));
                        } else {
                            break;
                        }
                    }
                },
                None => {
                    if res.is_empty() {
                        return Err((
                            LexErrorKind::InvalidListName {
                                name: res.into_boxed_str(),
                            },
                            input,
                        ));
                    } else {
                        break;
                    }
                }
            }
        }

        if res.as_bytes().first() == Some(&b'.') || res.as_bytes().last() == Some(&b'.') {
            return Err((
                LexErrorKind::InvalidListName {
                    name: res.into_boxed_str(),
                },
                input,
            ));
        }

        Ok((res.into(), rest))
    }
}

impl ListName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn valid() {
        assert_ok!(
            ListName::lex("$hello;"),
            ListName::from("hello".to_string()),
            ";"
        );

        assert_ok!(
            ListName::lex("$hello_world;"),
            ListName::from("hello_world".to_string()),
            ";"
        );

        assert_ok!(
            ListName::lex("$hello.world;"),
            ListName::from("hello.world".to_string()),
            ";"
        );

        assert_ok!(
            ListName::lex("$hello1234567890;"),
            ListName::from("hello1234567890".to_string()),
            ";"
        );

        assert_ok!(
            ListName::lex("$hello"),
            ListName::from("hello".to_string()),
            ""
        );
    }

    #[test]
    fn hyphens() {
        assert_ok!(
            ListName::lex("$01302951-49f9-47c9-a400-0297e60b6a10;"),
            ListName::from("01302951".to_string()),
            "-49f9-47c9-a400-0297e60b6a10;"
        );

        assert_ok!(
            ListName::lex_with_hyphens("$01302951-49f9-47c9-a400-0297e60b6a10;", true),
            ListName::from("01302951-49f9-47c9-a400-0297e60b6a10".to_string()),
            ";"
        );
    }

    #[test]
    fn invalid_char() {
        assert_err!(
            ListName::lex("$;"),
            LexErrorKind::InvalidListName { name: ";".into() },
            ";"
        );
    }

    #[test]
    fn eof_after_dollar() {
        assert_err!(
            ListName::lex("$"),
            LexErrorKind::InvalidListName { name: "".into() },
            ""
        );
    }

    #[test]
    fn no_dollar() {
        assert_err!(
            ListName::lex("abc"),
            LexErrorKind::ExpectedLiteral("$"),
            "abc"
        );
    }

    #[test]
    fn special_char_at_start() {
        assert_err!(
            ListName::lex("$."),
            LexErrorKind::InvalidListName { name: ".".into() },
            "."
        );

        assert_err!(
            ListName::lex("$.abc"),
            LexErrorKind::InvalidListName {
                name: ".abc".into(),
            },
            ".abc"
        );
    }

    #[test]
    fn special_char_at_end() {
        assert_err!(
            ListName::lex("$."),
            LexErrorKind::InvalidListName { name: ".".into() },
            "."
        );

        assert_err!(
            ListName::lex("$abc."),
            LexErrorKind::InvalidListName {
                name: "abc.".into(),
            },
            "abc."
        );
    }
}
