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
        println!("peek: {:?}", self.next_token);
        &self.next_token
    }
    pub fn pop_token(&mut self) -> Option<Token> {
        if self.next_token.is_none() {
            self.next_token = self.pop_token_internal();
        }
        println!("pop: {:?}", self.next_token);
        self.next_token.take()
    }
    pub fn consume(&mut self, token: Token) -> bool {
        if self.peek_token() == &Some(token) {
            self.next_token.take();
            true
        } else {
            false
        }
    }
    pub fn expect(&mut self, token: Token) {
        if self.peek_token().as_ref() == Some(&token) {
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
    Predicate {
        name: String,
        args: Vec<Rc<Node>>,
    },
    Clause {
        left: Rc<Node>,
        right: Vec<Rc<Node>>,
    },
    True,
}

struct Parser {
    lexer: Lexer,
}
impl Parser {
    fn new(lexer: Lexer) -> Self {
        Parser { lexer }
    }
    fn parse_atom(&mut self) -> Option<Node> {
        if let Some(Token::Atom(s)) = self.lexer.peek_token() {
            let s = s.clone();
            self.lexer.pop_token();
            Some(Node::Atom(s))
        } else {
            None
        }
    }
    fn parse_term(&mut self) -> Option<Node> {
        let token = self.lexer.pop_token();
        match token {
            Some(Token::Atom(s)) => Some(Node::Atom(s)),
            Some(Token::Variable(s)) => Some(Node::Variable(s)),
            _ => {
                panic!("Expected term but got {:?}", token)
            }
        }
    }
    fn parse_predicate(&mut self) -> Option<Node> {
        if let Node::Atom(name) = self.parse_atom()? {
            if !self.lexer.consume(Token::Op("(".to_string())) {
                Some(Node::Predicate {
                    name,
                    args: Vec::new(),
                })
            } else {
                let args = self.parse_term_list()?;
                self.lexer.expect(Token::Op(")".to_string()));
                Some(Node::Predicate { name, args })
            }
        } else {
            None
        }
    }
    fn parse_predicate_list(&mut self) -> Option<Vec<Rc<Node>>> {
        let p = self.parse_predicate()?;
        let mut plist = vec![Rc::new(p)];
        while self.lexer.consume(Token::Op(",".to_string())) {
            let p = self.parse_predicate()?;
            plist.push(Rc::new(p));
        }
        Some(plist)
    }
    fn parse_term_list(&mut self) -> Option<Vec<Rc<Node>>> {
        let p = self.parse_term()?;
        let mut plist = vec![Rc::new(p)];
        while self.lexer.consume(Token::Op(",".to_string())) {
            let p = self.parse_term()?;
            plist.push(Rc::new(p));
        }
        Some(plist)
    }
    fn parse_clause(&mut self) -> Option<Node> {
        if let Some(left) = self.parse_predicate() {
            if self.lexer.consume(Token::Op(":-".to_string())) {
                if let Some(right) = self.parse_predicate_list() {
                    Some(Node::Clause {
                        left: Rc::new(left),
                        right,
                    })
                } else {
                    panic!("Expected predicate but got {:?}", self.lexer.peek_token())
                }
            } else {
                Some(left)
            }
        } else {
            None
        }
    }
    fn parse_rule(&mut self) -> Option<Node> {
        match self.parse_clause() {
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
        println!("input: {}", input);
        let lexer = Lexer::new(input.to_string());
        let mut parser = Parser::new(lexer);
        let node = parser.parse();
        println!("node: {:?}", node);
        node
    }
    fn op(s: &str) -> Token {
        Token::Op(s.to_string())
    }

    #[test]
    fn parse() {
        use crate::Node::*;
        assert!(
            parse_input("eq(a).")
                == vec![Predicate {
                    name: "eq".to_string(),
                    args: vec![Rc::new(Atom("a".to_string()))]
                }]
        );
        assert!(
            parse_input("eq(A).")
                == vec![Predicate {
                    name: "eq".to_string(),
                    args: vec![Rc::new(Variable("A".to_string()))]
                }]
        );
        assert!(
            parse_input("cat.")
                == vec![Predicate {
                    name: "cat".to_string(),
                    args: Vec::new()
                }]
        );
        assert!(
            parse_input("cat :- true.")
                == vec![Clause {
                    left: Rc::new(Predicate {
                        name: "cat".to_string(),
                        args: Vec::new()
                    }),
                    right: vec![Rc::new(Predicate {
                        name: "true".to_string(),
                        args: Vec::new()
                    })],
                }]
        );
        assert!(
            parse_input("a :- b(X).")
                == vec![Clause {
                    left: Rc::new(Predicate {
                        name: "a".to_string(),
                        args: Vec::new()
                    }),
                    right: vec![Rc::new(Predicate {
                        name: "b".to_string(),
                        args: vec![Rc::new(Variable("X".to_string()))],
                    })],
                }]
        );
        assert!(
            parse_input("add(X, e, X).")
                == vec![Predicate {
                    name: "add".to_string(),
                    args: vec![
                        Rc::new(Variable("X".to_string())),
                        Rc::new(Atom("e".to_string())),
                        Rc::new(Variable("X".to_string())),
                    ],
                },]
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
