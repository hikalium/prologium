use std::rc::Rc;

#[derive(Debug, PartialEq)]
pub enum Token {
    Atom(String),
    Variable(String),
    Op(String),
}

pub struct Lexer {
    pos: usize,
    input: String,
    next_token: Option<Token>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            pos: 0,
            input,
            next_token: None,
        }
    }

    pub fn peek(&self) -> Option<char> {
        self.input.chars().nth(self.pos)
    }

    pub fn pop(&mut self) -> Option<char> {
        self.pos += 1;
        self.input.chars().nth(self.pos - 1)
    }

    fn pop_token_internal(&mut self) -> Option<Token> {
        loop {
            match self.peek() {
                None => return None,
                Some(c) => match c {
                    'a'..='z' => {
                        let mut s = String::new();
                        while let Some(c @ 'a'..='z') = self.peek() {
                            self.pop();
                            s.push(c);
                        }
                        return Some(Token::Atom(s));
                    }
                    c @ 'A'..='Z' => {
                        let mut s = String::new();
                        self.pop();
                        s.push(c);
                        while let Some(c @ 'a'..='z') = self.peek() {
                            self.pop();
                            s.push(c);
                        }
                        return Some(Token::Variable(s));
                    }
                    c @ ('(' | ')' | ',' | '.') => {
                        self.pop();
                        return Some(Token::Op(c.to_string()));
                    }
                    ':' => {
                        self.pop();
                        if let Some('-') = self.pop() {
                            return Some(Token::Op(":-".to_string()));
                        } else {
                            panic!("Expected -");
                        }
                    }
                    '%' => loop {
                        match self.pop() {
                            Some('\n') => break,
                            Some(_) => {}
                            None => break,
                        }
                    },
                    '\n' | ' ' => {
                        self.pop();
                    }
                    c => panic!("Unexpected char {}", c),
                },
            }
        }
    }
    pub fn peek_token(&mut self) -> &Option<Token> {
        if self.next_token.is_none() {
            self.next_token = self.pop_token_internal();
        }
        &self.next_token
    }
    pub fn pop_token(&mut self) -> Option<Token> {
        if self.next_token.is_none() {
            self.next_token = self.pop_token_internal();
        }
        self.next_token.take()
    }
    pub fn consume(&mut self, token: Token) -> bool {
        if self.next_token.is_none() {
            self.next_token = self.pop_token_internal();
        }
        if self.next_token == Some(token) {
            self.next_token.take();
            true
        } else {
            false
        }
    }
    pub fn expect(&mut self, token: Token) {
        if self.next_token.is_none() {
            self.next_token = self.pop_token_internal();
        }
        if self.next_token.as_ref() == Some(&token) {
            self.next_token.take();
        } else {
            panic!("Expected {:?}", token);
        }
    }
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.pop_token()
    }
}

#[derive(Debug, PartialEq)]
pub enum Node {
    Atom(String),
    Variable(String),
    Rule { left: Rc<Node>, right: Rc<Node> },
    True,
}

struct Parser {
    lexer: Lexer,
}
impl Parser {
    fn new(lexer: Lexer) -> Self {
        Parser { lexer }
    }
    fn parse_fact(&mut self) -> Option<Node> {
        let node = match self.lexer.peek_token() {
            Some(Token::Atom(s)) => Some(Node::Atom(s.to_string())),
            Some(Token::Variable(s)) => Some(Node::Variable(s.to_string())),
            _ => {
                return None;
            }
        };
        self.lexer.pop_token();
        node
    }
    fn parse_predicate(&mut self) -> Option<Node> {
        if let Some(left) = self.parse_fact() {
            if !self.lexer.consume(Token::Op(":-".to_string())) {
                Some(left)
            } else {
                if let Some(e) = self.parse_fact() {
                    Some(Node::Rule {
                        left: Rc::new(left),
                        right: Rc::new(e),
                    })
                } else {
                    Some(Node::Rule {
                        left: Rc::new(left),
                        right: Rc::new(Node::True),
                    })
                }
            }
        } else {
            None
        }
    }
    fn parse_rule(&mut self) -> Option<Node> {
        match self.parse_predicate() {
            Some(node) => {
                self.lexer.expect(Token::Op(".".to_string()));
                Some(node)
            }
            None => None,
        }
    }
    fn parse(&mut self) -> Vec<Node> {
        let mut nodes = Vec::new();
        loop {
            let node = self.parse_rule();
            match node {
                None => break,
                Some(node) => nodes.push(node),
            }
        }
        nodes
    }
}

fn main() -> std::io::Result<()> {
    loop {
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input)? == 0 {
            println!("EOF");
            break;
        }
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        let nodes = parser.parse();
        for node in nodes {
            println!("{:?}", node);
        }
        println!("OK");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_input(input: &str) -> Vec<Node> {
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        parser.parse()
    }
    fn op(s: &str) -> Token {
        Token::Op(s.to_string())
    }

    #[test]
    fn parse() {
        use crate::Node::*;
        assert!(parse_input("cat.") == vec![Atom("cat".to_string())]);
        assert!(parse_input("Cat.") == vec![Variable("Cat".to_string())]);
        assert!(
            parse_input("cat :- .")
                == vec![Rule {
                    left: Atom("cat".to_string()).into(),
                    right: True.into()
                }]
        );
        assert!(
            parse_input("a :- b.")
                == vec![Rule {
                    left: Atom("a".to_string()).into(),
                    right: Atom("b".to_string()).into(),
                }]
        );
    }

    #[test]
    fn tokenize() {
        use crate::Token::*;
        let tokens = Lexer::new("daughter(X, Y) :- father(Y, X), female(X). % comment".to_string());
        let tokens = tokens.collect::<Vec<Token>>();
        println!("{:?}", tokens);
        assert!(
            tokens
                == vec![
                    Atom("daughter".to_string()),
                    op("("),
                    Variable("X".to_string()),
                    op(","),
                    Variable("Y".to_string()),
                    op(")"),
                    op(":-"),
                    Atom("father".to_string()),
                    op("("),
                    Variable("Y".to_string()),
                    op(","),
                    Variable("X".to_string()),
                    op(")"),
                    op(","),
                    Atom("female".to_string()),
                    op("("),
                    Variable("X".to_string()),
                    op(")"),
                    op("."),
                ],
        )
    }
}
