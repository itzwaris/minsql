use crate::language::ast::*;
use crate::language::lexer::{Lexer, Token};
use anyhow::{Context, Result};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            position: 0,
        }
    }

    pub fn parse(&mut self, input: &str) -> Result<Statement> {
        let mut lexer = Lexer::new(input);
        self.tokens = lexer.tokenize()?;
        self.position = 0;

        self.parse_statement()
    }

    fn parse_statement(&mut self) -> Result<Statement> {
        match self.current()? {
            Token::Retrieve | Token::Select => self.parse_retrieve(),
            Token::Insert => self.parse_insert(),
            Token::Update => self.parse_update(),
            Token::Delete => self.parse_delete(),
            Token::Create => self.parse_create(),
            Token::Drop => self.parse_drop(),
            Token::Begin => self.parse_begin_transaction(),
            Token::Commit => {
                self.advance();
                Ok(Statement::Commit)
            }
            Token::Rollback => {
                self.advance();
                Ok(Statement::Rollback)
            }
            _ => anyhow::bail!("Unexpected token: {:?}", self.current()?),
        }
    }

    fn parse_retrieve(&mut self) -> Result<Statement> {
        self.advance();

        let projection = self.parse_projection()?;

        self.expect(Token::From)?;
        let from = self.parse_table_reference()?;

        let mut joins = Vec::new();
        while matches!(self.current(), Ok(Token::Join) | Ok(Token::Left) | Ok(Token::Inner)) {
            joins.push(self.parse_join()?);
        }

        let filter = if matches!(self.current(), Ok(Token::Where)) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        let group_by = if matches!(self.current(), Ok(Token::Group)) {
            self.advance();
            self.expect(Token::By)?;
            self.parse_expression_list()?
        } else {
            Vec::new()
        };

        let order_by = if matches!(self.current(), Ok(Token::Order)) {
            self.advance();
            self.expect(Token::By)?;
            self.parse_order_by_list()?
        } else {
            Vec::new()
        };

        let limit = if matches!(self.current(), Ok(Token::Limit)) {
            self.advance();
            Some(self.parse_integer()? as usize)
        } else {
            None
        };

        let offset = if matches!(self.current(), Ok(Token::Offset)) {
            self.advance();
            Some(self.parse_integer()? as usize)
        } else {
            None
        };

        let (at_timestamp, until_timestamp) = if matches!(self.current(), Ok(Token::At)) {
            self.advance();
            self.expect(Token::Timestamp)?;
            let at = Some(self.parse_string()?);
            
            let until = if matches!(self.current(), Ok(Token::Until)) {
                self.advance();
                self.expect(Token::Timestamp)?;
                Some(self.parse_string()?)
            } else {
                None
            };
            
            (at, until)
        } else {
            (None, None)
        };

        Ok(Statement::Retrieve(RetrieveStatement {
            projection,
            from,
            joins,
            filter,
            group_by,
            order_by,
            limit,
            offset,
            at_timestamp,
            until_timestamp,
        }))
    }

    fn parse_insert(&mut self) -> Result<Statement> {
        self.advance();
        self.expect(Token::Into)?;

        let table = self.parse_identifier()?;

        self.expect(Token::LeftParen)?;
        let columns = self.parse_identifier_list()?;
        self.expect(Token::RightParen)?;

        self.expect(Token::Values)?;

        let mut values = Vec::new();
        loop {
            self.expect(Token::LeftParen)?;
            let row = self.parse_expression_list()?;
            self.expect(Token::RightParen)?;
            values.push(row);

            if !matches!(self.current(), Ok(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(Statement::Insert(InsertStatement {
            table,
            columns,
            values,
        }))
    }

    fn parse_update(&mut self) -> Result<Statement> {
        self.advance();

        let table = self.parse_identifier()?;

        self.expect(Token::Set)?;

        let mut assignments = Vec::new();
        loop {
            let column = self.parse_identifier()?;
            self.expect(Token::Equals)?;
            let value = self.parse_expression()?;

            assignments.push(Assignment { column, value });

            if !matches!(self.current(), Ok(Token::Comma)) {
                break;
            }
            self.advance();
        }

        let filter = if matches!(self.current(), Ok(Token::Where)) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Update(UpdateStatement {
            table,
            assignments,
            filter,
        }))
    }

    fn parse_delete(&mut self) -> Result<Statement> {
        self.advance();
        self.expect(Token::From)?;

        let table = self.parse_identifier()?;

        let filter = if matches!(self.current(), Ok(Token::Where)) {
            self.advance();
            Some(self.parse_expression()?)
        } else {
            None
        };

        Ok(Statement::Delete(DeleteStatement { table, filter }))
    }

    fn parse_create(&mut self) -> Result<Statement> {
        self.advance();

        match self.current()? {
            Token::Table => self.parse_create_table(),
            Token::Index => self.parse_create_index(),
            _ => anyhow::bail!("Expected TABLE or INDEX after CREATE"),
        }
    }

    fn parse_create_table(&mut self) -> Result<Statement> {
        self.advance();

        let name = self.parse_identifier()?;

        self.expect(Token::LeftParen)?;

        let mut columns = Vec::new();
        loop {
            let col_name = self.parse_identifier()?;
            let data_type = self.parse_data_type()?;

            let mut nullable = true;
            let mut primary_key = false;

            columns.push(ColumnDefinition {
                name: col_name,
                data_type,
                nullable,
                primary_key,
            });

            if !matches!(self.current(), Ok(Token::Comma)) {
                break;
            }
            self.advance();
        }

        self.expect(Token::RightParen)?;

        Ok(Statement::CreateTable(CreateTableStatement { name, columns }))
    }

    fn parse_create_index(&mut self) -> Result<Statement> {
        self.advance();

        let name = self.parse_identifier()?;

        self.expect(Token::On)?;
        let table = self.parse_identifier()?;

        self.expect(Token::LeftParen)?;
        let columns = self.parse_identifier_list()?;
        self.expect(Token::RightParen)?;

        Ok(Statement::CreateIndex(CreateIndexStatement {
            name,
            table,
            columns,
        }))
    }

    fn parse_drop(&mut self) -> Result<Statement> {
        self.advance();
        self.expect(Token::Table)?;

        let name = self.parse_identifier()?;

        Ok(Statement::DropTable(DropTableStatement { name }))
    }

    fn parse_begin_transaction(&mut self) -> Result<Statement> {
        self.advance();

        let deterministic = if matches!(self.current(), Ok(Token::Deterministic)) {
            self.advance();
            true
        } else {
            false
        };

        self.expect(Token::Transaction)?;

        let at_timestamp = if matches!(self.current(), Ok(Token::At)) {
            self.advance();
            self.expect(Token::Timestamp)?;
            Some(self.parse_string()?)
        } else {
            None
        };

        Ok(Statement::BeginTransaction(BeginTransactionStatement {
            deterministic,
            at_timestamp,
        }))
    }

    fn parse_projection(&mut self) -> Result<Vec<Expression>> {
        if matches!(self.current(), Ok(Token::Star)) {
            self.advance();
            return Ok(vec![Expression::Star]);
        }

        self.parse_expression_list()
    }

    fn parse_expression_list(&mut self) -> Result<Vec<Expression>> {
        let mut expressions = Vec::new();

        loop {
            expressions.push(self.parse_expression()?);

            if !matches!(self.current(), Ok(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(expressions)
    }

    fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_or_expression()
    }

    fn parse_or_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_and_expression()?;

        while matches!(self.current(), Ok(Token::Or)) {
            self.advance();
            let right = self.parse_and_expression()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::Or,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_and_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_comparison_expression()?;

        while matches!(self.current(), Ok(Token::And)) {
            self.advance();
            let right = self.parse_comparison_expression()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::And,
                left: Box::new(left),
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    fn parse_comparison_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_additive_expression()?;

        if let Ok(token) = self.current() {
            let op = match token {
                Token::Equals => Some(BinaryOperator::Equals),
                Token::NotEquals => Some(BinaryOperator::NotEquals),
                Token::LessThan => Some(BinaryOperator::LessThan),
                Token::LessThanOrEqual => Some(BinaryOperator::LessThanOrEqual),
                Token::GreaterThan => Some(BinaryOperator::GreaterThan),
                Token::GreaterThanOrEqual => Some(BinaryOperator::GreaterThanOrEqual),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_additive_expression()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
        }

        Ok(left)
    }

    fn parse_additive_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_multiplicative_expression()?;

        while let Ok(token) = self.current() {
            let op = match token {
                Token::Plus => Some(BinaryOperator::Add),
                Token::Minus => Some(BinaryOperator::Subtract),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_multiplicative_expression()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_multiplicative_expression(&mut self) -> Result<Expression> {
        let mut left = self.parse_unary_expression()?;

        while let Ok(token) = self.current() {
            let op = match token {
                Token::Star => Some(BinaryOperator::Multiply),
                Token::Divide => Some(BinaryOperator::Divide),
                _ => None,
            };

            if let Some(op) = op {
                self.advance();
                let right = self.parse_unary_expression()?;
                left = Expression::BinaryOp {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            } else {
                break;
            }
        }

        Ok(left)
    }

    fn parse_unary_expression(&mut self) -> Result<Expression> {
        match self.current()? {
            Token::Not => {
                self.advance();
                let operand = self.parse_unary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Not,
                    operand: Box::new(operand),
                })
            }
            Token::Minus => {
                self.advance();
                let operand = self.parse_unary_expression()?;
                Ok(Expression::UnaryOp {
                    op: UnaryOperator::Negate,
                    operand: Box::new(operand),
                })
            }
            _ => self.parse_primary_expression(),
        }
    }

    fn parse_primary_expression(&mut self) -> Result<Expression> {
        match self.current()? {
            Token::LeftParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(Token::RightParen)?;
                Ok(expr)
            }
            Token::Integer(n) => {
                let val = *n;
                self.advance();
                Ok(Expression::Literal(Literal::Integer(val)))
            }
            Token::Float(f) => {
                let val = *f;
                self.advance();
                Ok(Expression::Literal(Literal::Float(val)))
            }
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expression::Literal(Literal::String(val)))
            }
            Token::Identifier(name) => {
                let id = name.clone();
                self.advance();

                if matches!(self.current(), Ok(Token::LeftParen)) {
                    self.advance();
                    let args = if matches!(self.current(), Ok(Token::RightParen)) {
                        Vec::new()
                    } else {
                        self.parse_expression_list()?
                    };
                    self.expect(Token::RightParen)?;
                    Ok(Expression::FunctionCall { name: id, args })
                } else if matches!(self.current(), Ok(Token::Dot)) {
                    self.advance();
                    let column = self.parse_identifier()?;
                    Ok(Expression::QualifiedColumn { table: id, column })
                } else {
                    Ok(Expression::Column(id))
                }
            }
            Token::Star => {
                self.advance();
                Ok(Expression::Star)
            }
            _ => anyhow::bail!("Unexpected token in expression: {:?}", self.current()?),
        }
    }

    fn parse_table_reference(&mut self) -> Result<TableReference> {
        let table = self.parse_identifier()?;

        if matches!(self.current(), Ok(Token::As)) {
            self.advance();
            let alias = self.parse_identifier()?;
            Ok(TableReference::Alias { table, alias })
        } else {
            Ok(TableReference::Table(table))
        }
    }

    fn parse_join(&mut self) -> Result<JoinClause> {
        let join_type = match self.current()? {
            Token::Inner => {
                self.advance();
                JoinType::Inner
            }
            Token::Left => {
                self.advance();
                if matches!(self.current(), Ok(Token::Outer)) {
                    self.advance();
                }
                JoinType::Left
            }
            _ => JoinType::Inner,
        };

        self.expect(Token::Join)?;

        let table = self.parse_table_reference()?;

        self.expect(Token::On)?;
        let on = self.parse_expression()?;

        Ok(JoinClause {
            join_type,
            table,
            on,
        })
    }

    fn parse_order_by_list(&mut self) -> Result<Vec<OrderByClause>> {
        let mut clauses = Vec::new();

        loop {
            let expr = self.parse_expression()?;
            let ascending = true;

            clauses.push(OrderByClause { expr, ascending });

            if !matches!(self.current(), Ok(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(clauses)
    }

    fn parse_identifier_list(&mut self) -> Result<Vec<String>> {
        let mut identifiers = Vec::new();

        loop {
            identifiers.push(self.parse_identifier()?);

            if !matches!(self.current(), Ok(Token::Comma)) {
                break;
            }
            self.advance();
        }

        Ok(identifiers)
    }

    fn parse_data_type(&mut self) -> Result<DataType> {
        let type_name = self.parse_identifier()?;

        match type_name.to_lowercase().as_str() {
            "boolean" | "bool" => Ok(DataType::Boolean),
            "integer" | "int" => Ok(DataType::Integer),
            "bigint" => Ok(DataType::BigInt),
            "real" | "float" => Ok(DataType::Real),
            "double" => Ok(DataType::Double),
            "text" | "string" | "varchar" => Ok(DataType::Text),
            "timestamp" | "datetime" => Ok(DataType::Timestamp),
            _ => anyhow::bail!("Unknown data type: {}", type_name),
        }
    }

    fn parse_identifier(&mut self) -> Result<String> {
        match self.current()? {
            Token::Identifier(name) => {
                let id = name.clone();
                self.advance();
                Ok(id)
            }
            _ => anyhow::bail!("Expected identifier, got {:?}", self.current()?),
        }
    }

    fn parse_string(&mut self) -> Result<String> {
        match self.current()? {
            Token::String(s) => {
                let val = s.clone();
                self.advance();
                Ok(val)
            }
            _ => anyhow::bail!("Expected string, got {:?}", self.current()?),
        }
    }

    fn parse_integer(&mut self) -> Result<i64> {
        match self.current()? {
            Token::Integer(n) => {
                let val = *n;
                self.advance();
                Ok(val)
            }
            _ => anyhow::bail!("Expected integer, got {:?}", self.current()?),
        }
    }

    fn current(&self) -> Result<&Token> {
        self.tokens
            .get(self.position)
            .context("Unexpected end of input")
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn expect(&mut self, expected: Token) -> Result<()> {
        let current = self.current()?.clone();
        
        let matches = match (&expected, &current) {
            (Token::Identifier(_), Token::Identifier(_)) => true,
            (Token::String(_), Token::String(_)) => true,
            (Token::Integer(_), Token::Integer(_)) => true,
            (Token::Float(_), Token::Float(_)) => true,
            _ => expected == current,
        };

        if matches {
            self.advance();
            Ok(())
        } else {
            anyhow::bail!("Expected {:?}, got {:?}", expected, current)
        }
    }
  }
