#[derive(Debug, PartialEq)]
pub enum Token {
    Atom(String),
    Variable(String),
    Op(char),
    OpAssign,
    Comment(String),
}

pub struct Lexer {
    pos: usize,
    input: String,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self { pos: 0, input }
    }

    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    pub fn pop(&mut self) -> Option<char> {
        self.pos += 1;
        self.input.chars().nth(self.pos - 1)
    }

    pub fn pop_token(&mut self) -> Option<Token> {
        // Skip white spaces
        while self.peek() == Some(' ') {
            self.pop();
        }
        match self.peek() {
            None => None,
            Some(c) => match c {
                'a'..='z' => {
                    let mut s = String::new();
                    while let Some(c @ 'a'..='z') = self.peek() {
                        self.pop();
                        s.push(c);
                    }
                    Some(Token::Atom(s))
                }
                c @ 'A'..='Z' => {
                    let mut s = String::new();
                    self.pop();
                    s.push(c);
                    while let Some(c @ 'a'..='z') = self.peek() {
                        self.pop();
                        s.push(c);
                    }
                    Some(Token::Variable(s))
                }
                c @ ('(' | ')' | ',' | '.') => {
                    self.pop();
                    Some(Token::Op(c))
                }
                ':' => {
                    self.pop();
                    if let Some('-') = self.pop() {
                        Some(Token::OpAssign)
                    } else {
                        panic!("Expected -");
                    }
                }
                '%' => {
                    let mut s = String::new();
                    loop {
                        match self.pop() {
                            Some('\n') => break,
                            Some(c) => {
                                s.push(c);
                            }
                            None => break,
                        }
                    }
                    Some(Token::Comment(s))
                }
                c => panic!("Unexpected char {}", c),
            },
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_token()
    }
}

fn main() -> std::io::Result<()> {
    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let tokens = Lexer::new(input);
        for t in tokens {
            println!("{:?}", t);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Token::*;

    #[test]
    fn tokenize() {
        let tokens = Lexer::new("daughter(X, Y) :- father(Y, X), female(X). % comment".to_string());
        let tokens = tokens.collect::<Vec<Token>>();
        println!("{:?}", tokens);
        assert!(
            tokens
                == vec![
                    Atom("daughter".to_string()),
                    Op('('),
                    Variable("X".to_string()),
                    Op(','),
                    Variable("Y".to_string()),
                    Op(')'),
                    OpAssign,
                    Atom("father".to_string()),
                    Op('('),
                    Variable("Y".to_string()),
                    Op(','),
                    Variable("X".to_string()),
                    Op(')'),
                    Op(','),
                    Atom("female".to_string()),
                    Op('('),
                    Variable("X".to_string()),
                    Op(')'),
                    Op('.'),
                    Comment("% comment".to_string()),
                ],
        )
    }
}
