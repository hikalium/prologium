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
                        return Some(Token::Op(c));
                    }
                    ':' => {
                        self.pop();
                        if let Some('-') = self.pop() {
                            return Some(Token::OpAssign);
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
}

struct Parser {
    lexer: Lexer,
}
impl Parser {
    fn new(lexer: Lexer) -> Self {
        Parser { lexer }
    }
    fn parse_fact(&mut self) -> Option<Node> {
        match self.lexer.next() {
            None => None,
            Some(Token::Atom(s)) => Some(Node::Atom(s)),
            Some(Token::Variable(s)) => Some(Node::Variable(s)),
            Some(t) => {
                panic!("Unexpected token {:?}", t);
            }
        }
    }
    fn parse_predicate(&mut self) -> Option<Node> {
        self.parse_fact()
    }
    fn parse_rule(&mut self) -> Option<Node> {
        match self.parse_predicate() {
            Some(node) => {
                match self.lexer.next() {
                    Some(Token::Op('.')) => {}
                    _ => {
                        panic!("Expected .");
                    }
                }
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

    #[test]
    fn parse() {
        use crate::Node::*;
        assert!(parse_input("cat.") == vec![Atom("cat".to_string())]);
        assert!(parse_input("Cat.") == vec![Variable("Cat".to_string())]);
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
                ],
        )
    }
}
