use crate::language::ast::Statement;
use crate::language::intent::*;
use crate::language::semantic::SemanticAnalyzer;
use crate::language::JoinType;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalPlan {
    Scan {
        table: String,
        columns: Vec<String>,
    },
    Filter {
        predicate: FilterIntent,
        input: Box<LogicalPlan>,
    },
    Project {
        columns: Vec<ColumnIntent>,
        input: Box<LogicalPlan>,
    },
    Join {
        join_type: JoinType,
        left: Box<LogicalPlan>,
        right: Box<LogicalPlan>,
        condition: FilterIntent,
    },
    Aggregate {
        group_by: Vec<ExpressionIntent>,
        aggregates: Vec<AggregateIntent>,
        input: Box<LogicalPlan>,
    },
    Sort {
        order_by: Vec<OrderIntent>,
        input: Box<LogicalPlan>,
    },
    Limit {
        count: usize,
        offset: usize,
        input: Box<LogicalPlan>,
    },
    Insert {
        table: String,
        columns: Vec<String>,
        values: Vec<Vec<ConstantValue>>,
    },
    Update {
        table: String,
        assignments: Vec<AssignmentIntent>,
        filter: Option<FilterIntent>,
    },
    Delete {
        table: String,
        filter: Option<FilterIntent>,
    },
}

pub struct LogicalPlanner {
    semantic_analyzer: SemanticAnalyzer,
}

impl LogicalPlanner {
    pub fn new() -> Self {
        Self {
            semantic_analyzer: SemanticAnalyzer::new(),
        }
    }

    pub fn plan(&self, statement: &Statement) -> Result<LogicalPlan> {
        let intent = self.semantic_analyzer.analyze(statement)?;
        self.intent_to_plan(&intent)
    }

    fn intent_to_plan(&self, intent: &Intent) -> Result<LogicalPlan> {
        match intent {
            Intent::Retrieve {
                columns,
                source,
                filter,
                aggregates,
                ordering,
                limit,
                ..
            } => {
                let mut plan = LogicalPlan::Scan {
                    table: source.primary.clone(),
                    columns: self.extract_column_names(columns),
                };

                for join in &source.joins {
                    let right = LogicalPlan::Scan {
                        table: join.table.clone(),
                        columns: vec![],
                    };

                    plan = LogicalPlan::Join {
                        join_type: join.join_type.clone(),
                        left: Box::new(plan),
                        right: Box::new(right),
                        condition: join.condition.clone(),
                    };
                }

                if let Some(filter_intent) = filter {
                    plan = LogicalPlan::Filter {
                        predicate: filter_intent.clone(),
                        input: Box::new(plan),
                    };
                }

                if !aggregates.is_empty() {
                    plan = LogicalPlan::Aggregate {
                        group_by: vec![],
                        aggregates: aggregates.clone(),
                        input: Box::new(plan),
                    };
                }

                plan = LogicalPlan::Project {
                    columns: columns.clone(),
                    input: Box::new(plan),
                };

                if !ordering.is_empty() {
                    plan = LogicalPlan::Sort {
                        order_by: ordering.clone(),
                        input: Box::new(plan),
                    };
                }

                if let Some(limit_count) = limit {
                    plan = LogicalPlan::Limit {
                        count: *limit_count,
                        offset: 0,
                        input: Box::new(plan),
                    };
                }

                Ok(plan)
            }
            Intent::Mutate {
                operation,
                target,
                filter,
            } => match operation {
                MutationIntent::Insert { columns, values } => Ok(LogicalPlan::Insert {
                    table: target.clone(),
                    columns: columns.clone(),
                    values: values.clone(),
                }),
                MutationIntent::Update { assignments } => Ok(LogicalPlan::Update {
                    table: target.clone(),
                    assignments: assignments.clone(),
                    filter: filter.clone(),
                }),
                MutationIntent::Delete => Ok(LogicalPlan::Delete {
                    table: target.clone(),
                    filter: filter.clone(),
                }),
            },
            _ => anyhow::bail!("Unsupported intent type"),
        }
    }

    fn extract_column_names(&self, columns: &[ColumnIntent]) -> Vec<String> {
        columns
            .iter()
            .filter_map(|col| match col {
                ColumnIntent::Named(name) => Some(name.clone()),
                ColumnIntent::Qualified { column, .. } => Some(column.clone()),
                _ => None,
            })
            .collect()
    }
                  }
