use crate::planner::physical::PhysicalPlan;
use crate::sharding::keyspace::ShardId;
use std::collections::HashSet;

pub struct LocalityAnalyzer;

impl LocalityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    pub fn analyze(&self, plan: &PhysicalPlan) -> LocalityInfo {
        match plan {
            PhysicalPlan::SeqScan { table, .. } => LocalityInfo {
                shards: self.get_table_shards(table),
                is_local: false,
                requires_shuffle: false,
            },
            PhysicalPlan::IndexScan { table, .. } => LocalityInfo {
                shards: self.get_table_shards(table),
                is_local: false,
                requires_shuffle: false,
            },
            PhysicalPlan::Filter { input, .. } => self.analyze(input),
            PhysicalPlan::Project { input, .. } => self.analyze(input),
            PhysicalPlan::HashJoin { left, right, .. } => {
                let left_info = self.analyze(left);
                let right_info = self.analyze(right);

                let requires_shuffle = !self.are_colocated(&left_info.shards, &right_info.shards);

                let mut shards = left_info.shards.clone();
                shards.extend(right_info.shards);

                LocalityInfo {
                    shards,
                    is_local: left_info.is_local && right_info.is_local,
                    requires_shuffle,
                }
            }
            _ => LocalityInfo {
                shards: HashSet::new(),
                is_local: true,
                requires_shuffle: false,
            },
        }
    }

    fn get_table_shards(&self, _table: &str) -> HashSet<ShardId> {
        let mut shards = HashSet::new();
        for i in 0..16 {
            shards.insert(ShardId(i));
        }
        shards
    }

    fn are_colocated(&self, left: &HashSet<ShardId>, right: &HashSet<ShardId>) -> bool {
        left == right
    }
}

#[derive(Debug, Clone)]
pub struct LocalityInfo {
    pub shards: HashSet<ShardId>,
    pub is_local: bool,
    pub requires_shuffle: bool,
}
