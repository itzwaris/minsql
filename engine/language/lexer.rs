use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Retrieve,
    Insert,
    Update,
    Delete,
    From,
    Where,
    Set,
    Values,
    Into,
    Create,
    Drop,
    Table,
    Index,
    On,
    At,
    Until,
    Timestamp,
    Begin,
    Commit,
    Rollback,
    Transaction,
    Deterministic,
    Join,
    Left,
    Inner,
    Outer,
    Group,
    By,
    Order,
    Limit,
    Offset,
    As,
    With,
    Select,
    
    Identifier(String),
    String(String),
    Integer(i64),
    Float(f64),
    
    LeftParen,
    RightParen,
    Comma,
    Semicolon,
    Dot,
    Star,
    
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    
    Plus,
    Minus,
    Multiply,
    Divide,
    
    And,
    Or,
    Not,
    
    Eof,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            if self.position >= self.input.len() {
                break;
            }

            if self.peek() == Some('-') && self.peek_next() == Some('-') {
                self.skip_line_comment();
                continue;
            }

            if self.peek() == Some('/') && self.peek_next() == Some('*') {
                self.skip_block_comment()?;
                continue;
            }

            let token = self.next_token()?;
            tokens.push(token);
        }

        tokens.push(Token::Eof);
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token> {
        let ch = self.current().context("Unexpected end of input")?;

        match ch {
            '(' => {
                self.advance();
                Ok(Token::LeftParen)
            }
            ')' => {
                self.advance();
                Ok(Token::RightParen)
            }
            ',' => {
                self.advance();
                Ok(Token::Comma)
            }
            ';' => {
                self.advance();
                Ok(Token::Semicolon)
            }
            '.' => {
                self.advance();
                Ok(Token::Dot)
            }
            '*' => {
                self.advance();
                Ok(Token::Star)
            }
            '+' => {
                self.advance();
                Ok(Token::Plus)
            }
            '-' => {
                self.advance();
                Ok(Token::Minus)
            }
            '/' => {
                self.advance();
                Ok(Token::Divide)
            }
            '=' => {
                self.advance();
                Ok(Token::Equals)
            }
            '<' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::LessThanOrEqual)
                } else if self.current() == Some('>') {
                    self.advance();
                    Ok(Token::NotEquals)
                } else {
                    Ok(Token::LessThan)
                }
            }
            '>' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::GreaterThanOrEqual)
                } else {
                    Ok(Token::GreaterThan)
                }
            }
            '!' => {
                self.advance();
                if self.current() == Some('=') {
                    self.advance();
                    Ok(Token::NotEquals)
                } else {
                    anyhow::bail!("Unexpected character: !")
                }
            }
            '\'' | '"' => self.read_string(),
            _ if ch.is_ascii_digit() => self.read_number(),
            _ if ch.is_ascii_alphabetic() || ch == '_' => self.read_identifier(),
            _ => anyhow::bail!("Unexpected character: {}", ch),
        }
    }

    fn read_identifier(&mut self) -> Result<Token> {
        let mut ident = String::new();

        while let Some(ch) = self.current() {
            if ch.is_ascii_alphanumeric() || ch == '_' {
                ident.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        let token = match ident.to_lowercase().as_str() {
            "retrieve" => Token::Retrieve,
            "insert" => Token::Insert,
            "update" => Token::Update,
            "delete" => Token::Delete,
            "from" => Token::From,
            "where" => Token::Where,
            "set" => Token::Set,
            "values" => Token::Values,
            "into" => Token::Into,
            "create" => Token::Create,
            "drop" => Token::Drop,
            "table" => Token::Table,
            "index" => Token::Index,
            "on" => Token::On,
            "at" => Token::At,
            "until" => Token::Until,
            "timestamp" => Token::Timestamp,
            "begin" => Token::Begin,
            "commit" => Token::Commit,
            "rollback" => Token::Rollback,
            "transaction" => Token::Transaction,
            "deterministic" => Token::Deterministic,
            "join" => Token::Join,
            "left" => Token::Left,
            "inner" => Token::Inner,
            "outer" => Token::Outer,
            "group" => Token::Group,
            "by" => Token::By,
            "order" => Token::Order,
            "limit" => Token::Limit,
            "offset" => Token::Offset,
            "as" => Token::As,
            "with" => Token::With,
            "select" => Token::Select,
            "and" => Token::And,
            "or" => Token::Or,
            "not" => Token::Not,
            _ => Token::Identifier(ident),
        };

        Ok(token)
    }

    fn read_number(&mut self) -> Result<Token> {
        let mut num = String::new();
        let mut is_float = false;

        while let Some(ch) = self.current() {
            if ch.is_ascii_digit() {
                num.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                is_float = true;
                num.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        if is_float {
            let val = num.parse::<f64>().context("Invalid float")?;
            Ok(Token::Float(val))
        } else {
            let val = num.parse::<i64>().context("Invalid integer")?;
            Ok(Token::Integer(val))
        }
    }

    fn read_string(&mut self) -> Result<Token> {
        let quote_char = self.current().unwrap();
        self.advance();

        let mut str_val = String::new();

        loop {
            match self.current() {
                Some(ch) if ch == quote_char => {
                    self.advance();
                    break;
                }
                Some(ch) => {
                    str_val.push(ch);
                    self.advance();
                }
                None => anyhow::bail!("Unterminated string"),
            }
        }

        Ok(Token::String(str_val))
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        while let Some(ch) = self.current() {
            self.advance();
            if ch == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        self.advance();
        self.advance();

        loop {
            match self.current() {
                Some('*') if self.peek_next() == Some('/') => {
                    self.advance();
                    self.advance();
                    break;
                }
                Some(_) => self.advance(),
                None => anyhow::bail!("Unterminated block comment"),
            }
        }

        Ok(())
    }

    fn current(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn peek(&self) -> Option<char> {
        if self.position < self.input.len() {
            Some(self.input[self.position])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.position + 1 < self.input.len() {
            Some(self.input[self.position + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }
              }
