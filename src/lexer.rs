use std::str::Chars;
use std::iter::Peekable;

pub struct Lexer<'a> {
    chars_it: Peekable<Chars<'a>>
}

impl<'a> Lexer<'a> {
    pub fn new(chars: &'a str) -> Self {
        Self {
            chars_it: chars.chars().peekable()
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
            char::is_numeric, char::is_alphanumeric
        ];

        self.take_while(|c| c.is_whitespace());

        for predicate in predicates {
            let result = self.take_while(|c| predicate(*c));
            if !result.is_empty() {
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