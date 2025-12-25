use crate::language::intent::*;
use crate::planner::logical::LogicalPlan;
use anyhow::Result;

pub struct Optimizer;

impl Optimizer {
    pub fn new() -> Self {
        Self
    }

    pub fn optimize(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        let plan = self.push_down_filters(plan)?;
        let plan = self.push_down_projections(plan)?;
        let plan = self.fold_constants(plan)?;
        Ok(plan)
    }

    fn push_down_filters(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        match plan {
            LogicalPlan::Filter { predicate, input } => {
                match *input {
                    LogicalPlan::Join {
                        join_type,
                        left,
                        right,
                        condition,
                    } => {
                        let optimized_left = self.push_down_filters(*left)?;
                        let optimized_right = self.push_down_filters(*right)?;

                        Ok(LogicalPlan::Filter {
                            predicate,
                            input: Box::new(LogicalPlan::Join {
                                join_type,
                                left: Box::new(optimized_left),
                                right: Box::new(optimized_right),
                                condition,
                            }),
                        })
                    }
                    _ => {
                        let optimized_input = self.push_down_filters(*input)?;
                        Ok(LogicalPlan::Filter {
                            predicate,
                            input: Box::new(optimized_input),
                        })
                    }
                }
            }
            LogicalPlan::Project { columns, input } => {
                let optimized_input = self.push_down_filters(*input)?;
                Ok(LogicalPlan::Project {
                    columns,
                    input: Box::new(optimized_input),
                })
            }
            LogicalPlan::Join {
                join_type,
                left,
                right,
                condition,
            } => {
                let optimized_left = self.push_down_filters(*left)?;
                let optimized_right = self.push_down_filters(*right)?;
                Ok(LogicalPlan::Join {
                    join_type,
                    left: Box::new(optimized_left),
                    right: Box::new(optimized_right),
                    condition,
                })
            }
            _ => Ok(plan),
        }
    }

    fn push_down_projections(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        match plan {
            LogicalPlan::Project { columns, input } => {
                let optimized_input = self.push_down_projections(*input)?;
                Ok(LogicalPlan::Project {
                    columns,
                    input: Box::new(optimized_input),
                })
            }
            LogicalPlan::Filter { predicate, input } => {
                let optimized_input = self.push_down_projections(*input)?;
                Ok(LogicalPlan::Filter {
                    predicate,
                    input: Box::new(optimized_input),
                })
            }
            _ => Ok(plan),
        }
    }

    fn fold_constants(&self, plan: LogicalPlan) -> Result<LogicalPlan> {
        match plan {
            LogicalPlan::Filter { predicate, input } => {
                let optimized_input = self.fold_constants(*input)?;
                Ok(LogicalPlan::Filter {
                    predicate,
                    input: Box::new(optimized_input),
                })
            }
            LogicalPlan::Project { columns, input } => {
                let optimized_input = self.fold_constants(*input)?;
                Ok(LogicalPlan::Project {
                    columns,
                    input: Box::new(optimized_input),
                })
            }
            _ => Ok(plan),
        }
    }
                          }
