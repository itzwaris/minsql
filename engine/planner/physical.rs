use crate::ffi::storage::StorageEngine;
use crate::language::intent::*;
use crate::planner::logical::LogicalPlan;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PhysicalPlan {
    SeqScan {
        table: String,
        columns: Vec<String>,
    },
    IndexScan {
        table: String,
        index: String,
        columns: Vec<String>,
    },
    Filter {
        predicate: FilterIntent,
        input: Box<PhysicalPlan>,
    },
    Project {
        columns: Vec<ColumnIntent>,
        input: Box<PhysicalPlan>,
    },
    NestedLoopJoin {
        join_type: JoinType,
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        condition: FilterIntent,
    },
    HashJoin {
        join_type: JoinType,
        left: Box<PhysicalPlan>,
        right: Box<PhysicalPlan>,
        condition: FilterIntent,
    },
    HashAggregate {
        group_by: Vec<ExpressionIntent>,
        aggregates: Vec<AggregateIntent>,
        input: Box<PhysicalPlan>,
    },
    Sort {
        order_by: Vec<OrderIntent>,
        input: Box<PhysicalPlan>,
    },
    Limit {
        count: usize,
        offset: usize,
        input: Box<PhysicalPlan>,
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

pub struct PhysicalPlanner<'a> {
    storage: &'a StorageEngine,
}

impl<'a> PhysicalPlanner<'a> {
    pub fn new(storage: &'a StorageEngine) -> Self {
        Self { storage }
    }

    pub fn plan(&self, logical_plan: &LogicalPlan) -> Result<PhysicalPlan> {
        match logical_plan {
            LogicalPlan::Scan { table, columns } => {
                Ok(PhysicalPlan::SeqScan {
                    table: table.clone(),
                    columns: columns.clone(),
                })
            }
            LogicalPlan::Filter { predicate, input } => {
                Ok(PhysicalPlan::Filter {
                    predicate: predicate.clone(),
                    input: Box::new(self.plan(input)?),
                })
            }
            LogicalPlan::Project { columns, input } => {
                Ok(PhysicalPlan::Project {
                    columns: columns.clone(),
                    input: Box::new(self.plan(input)?),
                })
            }
            LogicalPlan::Join {
                join_type,
                left,
                right,
                condition,
            } => {
                let left_plan = self.plan(left)?;
                let right_plan = self.plan(right)?;

                Ok(PhysicalPlan::HashJoin {
                    join_type: join_type.clone(),
                    left: Box::new(left_plan),
                    right: Box::new(right_plan),
                    condition: condition.clone(),
                })
            }
            LogicalPlan::Aggregate {
                group_by,
                aggregates,
                input,
            } => {
                Ok(PhysicalPlan::HashAggregate {
                    group_by: group_by.clone(),
                    aggregates: aggregates.clone(),
                    input: Box::new(self.plan(input)?),
                })
            }
            LogicalPlan::Sort { order_by, input } => {
                Ok(PhysicalPlan::Sort {
                    order_by: order_by.clone(),
                    input: Box::new(self.plan(input)?),
                })
            }
            LogicalPlan::Limit {
                count,
                offset,
                input,
            } => {
                Ok(PhysicalPlan::Limit {
                    count: *count,
                    offset: *offset,
                    input: Box::new(self.plan(input)?),
                })
            }
            LogicalPlan::Insert {
                table,
                columns,
                values,
            } => {
                Ok(PhysicalPlan::Insert {
                    table: table.clone(),
                    columns: columns.clone(),
                    values: values.clone(),
                })
            }
            LogicalPlan::Update {
                table,
                assignments,
                filter,
            } => {
                Ok(PhysicalPlan::Update {
                    table: table.clone(),
                    assignments: assignments.clone(),
                    filter: filter.clone(),
                })
            }
            LogicalPlan::Delete { table, filter } => {
                Ok(PhysicalPlan::Delete {
                    table: table.clone(),
                    filter: filter.clone(),
                })
            }
        }
    }
                  }
