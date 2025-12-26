use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Retrieve(RetrieveStatement),
    Insert(InsertStatement),
    Update(UpdateStatement),
    Delete(DeleteStatement),
    CreateTable(CreateTableStatement),
    CreateIndex(CreateIndexStatement),
    DropTable(DropTableStatement),
    BeginTransaction(BeginTransactionStatement),
    Commit,
    Rollback,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrieveStatement {
    pub projection: Vec<Expression>,
    pub from: TableReference,
    pub joins: Vec<JoinClause>,
    pub filter: Option<Expression>,
    pub group_by: Vec<Expression>,
    pub order_by: Vec<OrderByClause>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub at_timestamp: Option<String>,
    pub until_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertStatement {
    pub table: String,
    pub columns: Vec<String>,
    pub values: Vec<Vec<Expression>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatement {
    pub table: String,
    pub assignments: Vec<Assignment>,
    pub filter: Option<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub column: String,
    pub value: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteStatement {
    pub table: String,
    pub filter: Option<Expression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTableStatement {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: DataType,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Boolean,
    Integer,
    BigInt,
    Real,
    Double,
    Text,
    Timestamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIndexStatement {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropTableStatement {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeginTransactionStatement {
    pub deterministic: bool,
    pub at_timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TableReference {
    Table(String),
    Alias { table: String, alias: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinClause {
    pub join_type: JoinType,
    pub table: TableReference,
    pub on: Expression,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderByClause {
    pub expr: Expression,
    pub ascending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Column(String),
    QualifiedColumn {
        table: String,
        column: String,
    },
    Literal(Literal),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Star,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equals,
    NotEquals,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
}
