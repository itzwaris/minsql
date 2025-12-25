use crate::language::ast::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Intent {
    Retrieve {
        columns: Vec<ColumnIntent>,
        source: SourceIntent,
        filter: Option<FilterIntent>,
        aggregates: Vec<AggregateIntent>,
        ordering: Vec<OrderIntent>,
        limit: Option<usize>,
        time_travel: Option<TimeTravelIntent>,
    },
    Mutate {
        operation: MutationIntent,
        target: String,
        filter: Option<FilterIntent>,
    },
    Schema {
        operation: SchemaIntent,
    },
    Transaction {
        operation: TransactionIntent,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColumnIntent {
    All,
    Named(String),
    Qualified { table: String, column: String },
    Expression { expr: ExpressionIntent, alias: Option<String> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceIntent {
    pub primary: String,
    pub joins: Vec<JoinIntent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JoinIntent {
    pub join_type: JoinType,
    pub table: String,
    pub condition: FilterIntent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterIntent {
    Always,
    Never,
    Comparison {
        op: ComparisonOp,
        left: ExpressionIntent,
        right: ExpressionIntent,
    },
    Logical {
        op: LogicalOp,
        operands: Vec<FilterIntent>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOp {
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOp {
    And,
    Or,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExpressionIntent {
    Column(String),
    QualifiedColumn { table: String, column: String },
    Constant(ConstantValue),
    Arithmetic {
        op: ArithmeticOp,
        left: Box<ExpressionIntent>,
        right: Box<ExpressionIntent>,
    },
    Function {
        name: String,
        args: Vec<ExpressionIntent>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstantValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArithmeticOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregateIntent {
    pub function: String,
    pub argument: ExpressionIntent,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderIntent {
    pub expr: ExpressionIntent,
    pub ascending: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeTravelIntent {
    pub at_time: String,
    pub until_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationIntent {
    Insert {
        columns: Vec<String>,
        values: Vec<Vec<ConstantValue>>,
    },
    Update {
        assignments: Vec<AssignmentIntent>,
    },
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentIntent {
    pub column: String,
    pub value: ExpressionIntent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchemaIntent {
    CreateTable {
        name: String,
        columns: Vec<ColumnDefinition>,
    },
    CreateIndex {
        name: String,
        table: String,
        columns: Vec<String>,
    },
    DropTable {
        name: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionIntent {
    Begin {
        deterministic: bool,
        at_timestamp: Option<String>,
    },
    Commit,
    Rollback,
}
