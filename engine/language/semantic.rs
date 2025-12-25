use crate::language::ast::*;
use crate::language::intent::*;
use anyhow::Result;

pub struct SemanticAnalyzer;

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, statement: &Statement) -> Result<Intent> {
        match statement {
            Statement::Retrieve(stmt) => self.analyze_retrieve(stmt),
            Statement::Insert(stmt) => self.analyze_insert(stmt),
            Statement::Update(stmt) => self.analyze_update(stmt),
            Statement::Delete(stmt) => self.analyze_delete(stmt),
            Statement::CreateTable(stmt) => self.analyze_create_table(stmt),
            Statement::CreateIndex(stmt) => self.analyze_create_index(stmt),
            Statement::DropTable(stmt) => self.analyze_drop_table(stmt),
            Statement::BeginTransaction(stmt) => self.analyze_begin_transaction(stmt),
            Statement::Commit => Ok(Intent::Transaction {
                operation: TransactionIntent::Commit,
            }),
            Statement::Rollback => Ok(Intent::Transaction {
                operation: TransactionIntent::Rollback,
            }),
        }
    }

    fn analyze_retrieve(&self, stmt: &RetrieveStatement) -> Result<Intent> {
        let columns = self.analyze_projection(&stmt.projection)?;
        
        let source = SourceIntent {
            primary: self.extract_table_name(&stmt.from)?,
            joins: stmt
                .joins
                .iter()
                .map(|j| self.analyze_join(j))
                .collect::<Result<Vec<_>>>()?,
        };

        let filter = stmt
            .filter
            .as_ref()
            .map(|f| self.analyze_filter(f))
            .transpose()?;

        let aggregates = Vec::new();

        let ordering = stmt
            .order_by
            .iter()
            .map(|o| self.analyze_order_by(o))
            .collect::<Result<Vec<_>>>()?;

        let time_travel = if let Some(at) = &stmt.at_timestamp {
            Some(TimeTravelIntent {
                at_time: at.clone(),
                until_time: stmt.until_timestamp.clone(),
            })
        } else {
            None
        };

        Ok(Intent::Retrieve {
            columns,
            source,
            filter,
            aggregates,
            ordering,
            limit: stmt.limit,
            time_travel,
        })
    }

    fn analyze_insert(&self, stmt: &InsertStatement) -> Result<Intent> {
        let mut values = Vec::new();

        for row in &stmt.values {
            let mut row_values = Vec::new();
            for expr in row {
                row_values.push(self.extract_constant(expr)?);
            }
            values.push(row_values);
        }

        Ok(Intent::Mutate {
            operation: MutationIntent::Insert {
                columns: stmt.columns.clone(),
                values,
            },
            target: stmt.table.clone(),
            filter: None,
        })
    }

    fn analyze_update(&self, stmt: &UpdateStatement) -> Result<Intent> {
        let assignments = stmt
            .assignments
            .iter()
            .map(|a| Ok(AssignmentIntent {
                column: a.column.clone(),
                value: self.analyze_expression_intent(&a.value)?,
            }))
            .collect::<Result<Vec<_>>>()?;

        let filter = stmt
            .filter
            .as_ref()
            .map(|f| self.analyze_filter(f))
            .transpose()?;

        Ok(Intent::Mutate {
            operation: MutationIntent::Update { assignments },
            target: stmt.table.clone(),
            filter,
        })
    }

    fn analyze_delete(&self, stmt: &DeleteStatement) -> Result<Intent> {
        let filter = stmt
            .filter
            .as_ref()
            .map(|f| self.analyze_filter(f))
            .transpose()?;

        Ok(Intent::Mutate {
            operation: MutationIntent::Delete,
            target: stmt.table.clone(),
            filter,
        })
    }

    fn analyze_create_table(&self, stmt: &CreateTableStatement) -> Result<Intent> {
        Ok(Intent::Schema {
            operation: SchemaIntent::CreateTable {
                name: stmt.name.clone(),
                columns: stmt.columns.clone(),
            },
        })
    }

    fn analyze_create_index(&self, stmt: &CreateIndexStatement) -> Result<Intent> {
        Ok(Intent::Schema {
            operation: SchemaIntent::CreateIndex {
                name: stmt.name.clone(),
                table: stmt.table.clone(),
                columns: stmt.columns.clone(),
            },
        })
    }

    fn analyze_drop_table(&self, stmt: &DropTableStatement) -> Result<Intent> {
        Ok(Intent::Schema {
            operation: SchemaIntent::DropTable {
                name: stmt.name.clone(),
            },
        })
    }

    fn analyze_begin_transaction(&self, stmt: &BeginTransactionStatement) -> Result<Intent> {
        Ok(Intent::Transaction {
            operation: TransactionIntent::Begin {
                deterministic: stmt.deterministic,
                at_timestamp: stmt.at_timestamp.clone(),
            },
        })
    }

    fn analyze_projection(&self, projection: &[Expression]) -> Result<Vec<ColumnIntent>> {
        projection
            .iter()
            .map(|expr| match expr {
                Expression::Star => Ok(ColumnIntent::All),
                Expression::Column(name) => Ok(ColumnIntent::Named(name.clone())),
                Expression::QualifiedColumn { table, column } => Ok(ColumnIntent::Qualified {
                    table: table.clone(),
                    column: column.clone(),
                }),
                expr => Ok(ColumnIntent::Expression {
                    expr: self.analyze_expression_intent(expr)?,
                    alias: None,
                }),
            })
            .collect()
    }

    fn analyze_join(&self, join: &JoinClause) -> Result<JoinIntent> {
        Ok(JoinIntent {
            join_type: join.join_type.clone(),
            table: self.extract_table_name(&join.table)?,
            condition: self.analyze_filter(&join.on)?,
        })
    }

    fn analyze_filter(&self, expr: &Expression) -> Result<FilterIntent> {
        match expr {
            Expression::BinaryOp { op, left, right } => {
                let left_intent = self.analyze_expression_intent(left)?;
                let right_intent = self.analyze_expression_intent(right)?;

                match op {
                    BinaryOperator::Equals => Ok(FilterIntent::Comparison {
                        op: ComparisonOp::Equal,
                        left: left_intent,
                        right: right_intent,
                    }),
                    BinaryOperator::NotEquals => Ok(FilterIntent::Comparison {
                        op: ComparisonOp::NotEqual,
                        left: left_intent,
                        right: right_intent,
                    }),
                    BinaryOperator::LessThan => Ok(FilterIntent::Comparison {
                        op: ComparisonOp::LessThan,
                        left: left_intent,
                        right: right_intent,
                    }),
                    BinaryOperator::LessThanOrEqual => Ok(FilterIntent::Comparison {
                        op: ComparisonOp::LessThanOrEqual,
                        left: left_intent,
                        right: right_intent,
                    }),
                    BinaryOperator::GreaterThan => Ok(FilterIntent::Comparison {
                        op: ComparisonOp::GreaterThan,
                        left: left_intent,
                        right: right_intent,
                    }),
                    BinaryOperator::GreaterThanOrEqual => Ok(FilterIntent::Comparison {
                        op: ComparisonOp::GreaterThanOrEqual,
                        left: left_intent,
                        right: right_intent,
                    }),
                    BinaryOperator::And => Ok(FilterIntent::Logical {
                        op: LogicalOp::And,
                        operands: vec![
                            self.analyze_filter(left)?,
                            self.analyze_filter(right)?,
                        ],
                    }),
                    BinaryOperator::Or => Ok(FilterIntent::Logical {
                        op: LogicalOp::Or,
                        operands: vec![
                            self.analyze_filter(left)?,
                            self.analyze_filter(right)?,
                        ],
                    }),
                    _ => anyhow::bail!("Invalid operator in filter: {:?}", op),
                }
            }
            Expression::UnaryOp { op, operand } => match op {
                UnaryOperator::Not => Ok(FilterIntent::Logical {
                    op: LogicalOp::Not,
                    operands: vec![self.analyze_filter(operand)?],
                }),
                _ => anyhow::bail!("Invalid unary operator in filter"),
            },
            _ => anyhow::bail!("Invalid filter expression"),
        }
    }

    fn analyze_expression_intent(&self, expr: &Expression) -> Result<ExpressionIntent> {
        match expr {
            Expression::Column(name) => Ok(ExpressionIntent::Column(name.clone())),
            Expression::QualifiedColumn { table, column } => {
                Ok(ExpressionIntent::QualifiedColumn {
                    table: table.clone(),
                    column: column.clone(),
                })
            }
            Expression::Literal(lit) => Ok(ExpressionIntent::Constant(self.convert_literal(lit))),
            Expression::BinaryOp { op, left, right } => {
                let left_intent = self.analyze_expression_intent(left)?;
                let right_intent = self.analyze_expression_intent(right)?;

                let arith_op = match op {
                    BinaryOperator::Add => ArithmeticOp::Add,
                    BinaryOperator::Subtract => ArithmeticOp::Subtract,
                    BinaryOperator::Multiply => ArithmeticOp::Multiply,
                    BinaryOperator::Divide => ArithmeticOp::Divide,
                    _ => anyhow::bail!("Non-arithmetic operator in expression"),
                };

                Ok(ExpressionIntent::Arithmetic {
                    op: arith_op,
                    left: Box::new(left_intent),
                    right: Box::new(right_intent),
                })
            }
            Expression::FunctionCall { name, args } => {
                let arg_intents = args
                    .iter()
                    .map(|a| self.analyze_expression_intent(a))
                    .collect::<Result<Vec<_>>>()?;

                Ok(ExpressionIntent::Function {
                    name: name.clone(),
                    args: arg_intents,
                })
            }
            _ => anyhow::bail!("Unsupported expression type"),
        }
    }

    fn analyze_order_by(&self, order_by: &OrderByClause) -> Result<OrderIntent> {
        Ok(OrderIntent {
            expr: self.analyze_expression_intent(&order_by.expr)?,
            ascending: order_by.ascending,
        })
    }

    fn extract_table_name(&self, table_ref: &TableReference) -> Result<String> {
        match table_ref {
            TableReference::Table(name) => Ok(name.clone()),
            TableReference::Alias { table, .. } => Ok(table.clone()),
        }
    }

    fn extract_constant(&self, expr: &Expression) -> Result<ConstantValue> {
        match expr {
            Expression::Literal(lit) => Ok(self.convert_literal(lit)),
            _ => anyhow::bail!("Expected constant value"),
        }
    }

    fn convert_literal(&self, lit: &Literal) -> ConstantValue {
        match lit {
            Literal::Null => ConstantValue::Null,
            Literal::Boolean(b) => ConstantValue::Boolean(*b),
            Literal::Integer(i) => ConstantValue::Integer(*i),
            Literal::Float(f) => ConstantValue::Float(*f),
            Literal::String(s) => ConstantValue::String(s.clone()),
        }
    }
          }
