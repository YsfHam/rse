use std::str::Chars;
use std::iter::Peekable;

use rust_stemmers::{Algorithm, Stemmer};

pub struct Lexer<'a> {
    chars_it: Peekable<Chars<'a>>,
    stemmer: Stemmer,
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a str) -> Self {
        Self {
            chars_it: chars.chars().peekable(),
            stemmer: Stemmer::create(Algorithm::English),
        }
    }

    fn take_while<P>(&mut self, predicate: P) -> String 
    where
        P: Fn(&char) -> bool
    {
        let mut res = String::new();
        while let Some(c) = self.chars_it.next_if(|c| predicate(c)) {
            res.push(c);
        } 
        res.to_lowercase()
    }

    fn next_token(&mut self) -> Option<String> {

        let predicates = [
            char::is_numeric,
            |c: char| {c.is_alphanumeric() || c == '_'}
        ];

        self.take_while(|c| c.is_whitespace());

        for predicate in predicates {
            let result = self.take_while(|c| predicate(*c));

            if !result.is_empty() {
                
                if result.chars().all(|c| c.is_alphabetic()) {
                    return Some(self.stemmer.stem(&result).to_string());
                }

                return Some(result);
            }
        }

        self.chars_it.next().and_then(|c| Some(c.to_string()))
    }
 
}

impl<'a> Iterator for Lexer<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}