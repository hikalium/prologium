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
                        while let Some(c @ ('a'..='z' | '0'..='9')) = self.peek() {
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
                    self.lexer.expect(Token::Op(".".to_string()));
                    Some(Node::Clause {
                        left: Rc::new(left),
                        right,
                    })
                } else {
                    panic!("Expected predicate but got {:?}", self.lexer.peek_token())
                }
            } else {
                self.lexer.expect(Token::Op(".".to_string()));
                Some(left)
            }
        } else {
            None
        }
    }
    fn parse(&mut self) -> Vec<Rc<Node>> {
        let mut nodes = Vec::new();
        loop {
            let node = self.parse_clause();
            match node {
                None => break,
                Some(node) => nodes.push(Rc::new(node)),
            }
        }
        nodes
    }
}

fn built_in_clause_list() -> Vec<Rc<Node>> {
    let lexer = Lexer::new(
        r#"
red(xff0000).
green(x00ff00).
blue(x0000ff).
            "#
        .to_string(),
    );
    let mut parser = Parser::new(lexer);
    parser.parse()
}

struct Evaluator {
    clause_list: Vec<Rc<Node>>,
    query: Node,
}

impl Evaluator {
    fn new(clause_list: Vec<Rc<Node>>, query: Node) -> Self {
        Self { clause_list, query }
    }
    fn eval(&self) -> bool {
        panic!("eval!!");
    }
}

fn main() -> std::io::Result<()> {
    let clause_list = built_in_clause_list();
    println!("{:?}", clause_list);
    loop {
        println!("query?");
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input)? == 0 {
            println!("EOF");
            break;
        }
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        if let Some(query) = parser.parse_clause() {
            println!("query: {:?}", query);
            let evaluator = Evaluator::new(clause_list.clone(), query);
            let result = evaluator.eval();
            println!("result: {:?}", result);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_input(input: &str) -> Vec<Rc<Node>> {
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
    fn predicate(name: &str, args: Vec<Rc<Node>>) -> Rc<Node> {
        Rc::new(crate::Node::Predicate {
            name: name.to_string(),
            args,
        })
    }
    fn clause(left: Rc<Node>, right: Vec<Rc<Node>>) -> Rc<Node> {
        Rc::new(crate::Node::Clause { left, right })
    }
    fn variable(name: &str) -> Rc<Node> {
        Rc::new(crate::Node::Variable(name.to_string()))
    }
    fn atom(name: &str) -> Rc<Node> {
        Rc::new(crate::Node::Atom(name.to_string()))
    }
    #[test]
    fn parse() {
        assert!(parse_input("eq(a).") == vec![predicate("eq", vec![atom("a")])]);
        assert!(parse_input("eq(A).") == vec![predicate("eq", vec![variable("A")])]);
        assert!(parse_input("cat.") == vec![predicate("cat", Vec::new())]);
        assert!(
            parse_input("cat :- true.")
                == vec![clause(
                    predicate("cat", Vec::new()),
                    vec![predicate("true", Vec::new())],
                )]
        );
        assert!(
            parse_input("a :- b(X).")
                == vec![clause(
                    predicate("a", Vec::new()),
                    vec![predicate("b", vec![variable("X")])],
                )]
        );
        assert!(
            parse_input("add(X, e, X).")
                == vec![predicate(
                    "add",
                    vec![variable("X"), atom("e"), variable("X"),]
                ),]
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
