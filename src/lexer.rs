
#![derive(Debug, Clone, Copy)]
pub enum Token<'input> {
    LBrace,
    RBrace,
    LParen,
    RParen,
    Ident(&'input str),
    Number(usize, Option<usize>),
}

enum Delimeter {
    Brace,
}

pub struct Lexer<'input> {
    input: &'input str,
    index: usize,
    current_state: State,
    delimeters:
}

impl<'input> Lexer<'input> {
    fn new(input: &'input str) -> Self {
        Lexer {
            input,
            index: 0,
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Token<'input>;

    fn next(&mut self) -> Option<Token<'input>> {
        loop {
            
        }
    }
}
