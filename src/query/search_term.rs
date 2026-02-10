use std::{fmt::Display, str::FromStr};

use logos::Logos;
use regex::Regex;

use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum SearchTerm {
    Verbatim(String),
    Word(String),
    Regex(Regex),
}

impl Display for SearchTerm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use SearchTerm as S;
        match self {
            S::Verbatim(search_term) => write!(f, "\"{search_term}\""),
            S::Word(search_term) => write!(f, "{search_term}"),
            S::Regex(search_term) => write!(f, "/{search_term}/"),
        }
    }
}

impl FromStr for SearchTerm {
    type Err = QueryParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lexer = QueryToken::lexer(s);

        let Some(Ok(next_token)) = lexer.next() else {
            return Err(QueryParseError::SearchTermExpected(
                lexer.span().start,
                lexer.slice().to_owned(),
            ));
        };

        to_search_term(next_token, &lexer)
    }
}

impl SearchTerm {
    pub fn test(&self, content_string: &str) -> bool {
        match self {
            SearchTerm::Regex(regex) => regex.is_match(content_string),
            SearchTerm::Verbatim(term) => content_string.contains(term),
            SearchTerm::Word(word) => content_string.to_lowercase().contains(&word.to_lowercase()),
        }
    }

    // helper for texts
    pub fn test_text(&self, text: &Text) -> bool {
        text.lines.iter().any(|line| {
            line.spans
                .iter()
                .any(|span| self.test(span.content.as_ref()))
        })
    }
}

pub fn to_search_term(
    query_token: QueryToken,
    lexer: &logos::Lexer<'_, QueryToken>,
) -> Result<SearchTerm, QueryParseError> {
    match query_token {
        QueryToken::Regex => {
            let mut search_term = lexer.slice().to_owned();
            strip_first_and_last(&mut search_term);

            let regex = regex::Regex::new(&search_term)?;
            Ok(SearchTerm::Regex(regex))
        }

        QueryToken::QuotedString => {
            let mut search_term = lexer.slice().to_owned();
            strip_first_and_last(&mut search_term);
            Ok(SearchTerm::Verbatim(search_term))
        }

        QueryToken::Word => {
            let search_term = lexer.slice().to_owned();
            Ok(SearchTerm::Word(search_term))
        }

        _ => Err(QueryParseError::SearchTermExpected(
            lexer.span().start,
            lexer.slice().to_owned(),
        )),
    }
}
