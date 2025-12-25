use crate::planner::physical::PhysicalPlan;

pub const PAGE_SCAN_COST: f64 = 1.0;
pub const CPU_TUPLE_COST: f64 = 0.01;
pub const CPU_OPERATOR_COST: f64 = 0.0025;

#[derive(Debug, Clone)]
pub struct Cost {
    pub cpu: f64,
    pub io: f64,
    pub memory: f64,
    pub network: f64,
}

impl Cost {
    pub fn total(&self) -> f64 {
        self.cpu + self.io + self.memory + self.network
    }

    pub fn zero() -> Self {
        Self {
            cpu: 0.0,
            io: 0.0,
            memory: 0.0,
            network: 0.0,
        }
    }
}

pub struct CostEstimator;

impl CostEstimator {
    pub fn new() -> Self {
        Self
    }

    pub fn estimate(&self, plan: &PhysicalPlan) -> Cost {
        match plan {
            PhysicalPlan::SeqScan { .. } => {
                let estimated_rows = 1000.0;
                Cost {
                    cpu: estimated_rows * CPU_TUPLE_COST,
                    io: estimated_rows * PAGE_SCAN_COST / 100.0,
                    memory: 0.0,
                    network: 0.0,
                }
            }
            PhysicalPlan::IndexScan { .. } => {
                let estimated_rows = 100.0;
                Cost {
                    cpu: estimated_rows * CPU_TUPLE_COST,
                    io: estimated_rows * PAGE_SCAN_COST / 100.0,
                    memory: 0.0,
                    network: 0.0,
                }
            }
            PhysicalPlan::Filter { input, .. } => {
                let input_cost = self.estimate(input);
                Cost {
                    cpu: input_cost.cpu + 1000.0 * CPU_OPERATOR_COST,
                    io: input_cost.io,
                    memory: input_cost.memory,
                    network: input_cost.network,
                }
            }
            PhysicalPlan::Project { input, .. } => {
                let input_cost = self.estimate(input);
                Cost {
                    cpu: input_cost.cpu + 1000.0 * CPU_OPERATOR_COST,
                    io: input_cost.io,
                    memory: input_cost.memory,
                    network: input_cost.network,
                }
            }
            PhysicalPlan::HashJoin { left, right, .. } => {
                let left_cost = self.estimate(left);
                let right_cost = self.estimate(right);
                Cost {
                    cpu: left_cost.cpu + right_cost.cpu + 10000.0 * CPU_OPERATOR_COST,
                    io: left_cost.io + right_cost.io,
                    memory: 1000.0,
                    network: 0.0,
                }
            }
            PhysicalPlan::NestedLoopJoin { left, right, .. } => {
                let left_cost = self.estimate(left);
                let right_cost = self.estimate(right);
                Cost {
                    cpu: left_cost.cpu + 1000.0 * right_cost.cpu,
                    io: left_cost.io + 1000.0 * right_cost.io,
                    memory: 0.0,
                    network: 0.0,
                }
            }
            PhysicalPlan::HashAggregate { input, .. } => {
                let input_cost = self.estimate(input);
                Cost {
                    cpu: input_cost.cpu + 1000.0 * CPU_OPERATOR_COST,
                    io: input_cost.io,
                    memory: 500.0,
                    network: 0.0,
                }
            }
            PhysicalPlan::Sort { input, .. } => {
                let input_cost = self.estimate(input);
                Cost {
                    cpu: input_cost.cpu + 5000.0 * CPU_OPERATOR_COST,
                    io: input_cost.io,
                    memory: 1000.0,
                    network: 0.0,
                }
            }
            PhysicalPlan::Limit { input, .. } => {
                let input_cost = self.estimate(input);
                Cost {
                    cpu: input_cost.cpu * 0.1,
                    io: input_cost.io * 0.1,
                    memory: input_cost.memory,
                    network: input_cost.network,
                }
            }
            _ => Cost::zero(),
        }
    }
              }
