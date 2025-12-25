use crate::sharding::keyspace::ShardId;
use std::collections::HashMap;

pub struct PlacementStrategy {
    pub shard_to_node: HashMap<ShardId, u32>,
    pub node_loads: HashMap<u32, usize>,
}

impl PlacementStrategy {
    pub fn new() -> Self {
        Self {
            shard_to_node: HashMap::new(),
            node_loads: HashMap::new(),
        }
    }

    pub fn assign_shard(&mut self, shard_id: ShardId, num_nodes: u32) -> u32 {
        let node_id = self.find_least_loaded_node(num_nodes);
        self.shard_to_node.insert(shard_id, node_id);
        *self.node_loads.entry(node_id).or_insert(0) += 1;
        node_id
    }

    pub fn get_node(&self, shard_id: ShardId) -> Option<u32> {
        self.shard_to_node.get(&shard_id).copied()
    }

    fn find_least_loaded_node(&self, num_nodes: u32) -> u32 {
        let mut min_load = usize::MAX;
        let mut selected_node = 0;

        for node_id in 0..num_nodes {
            let load = self.node_loads.get(&node_id).copied().unwrap_or(0);
            if load < min_load {
                min_load = load;
                selected_node = node_id;
            }
        }

        selected_node
    }

    pub fn rebalance(&mut self, num_nodes: u32) {
        let target_load = self.shard_to_node.len() / num_nodes as usize;

        for node_id in 0..num_nodes {
            let load = self.node_loads.get(&node_id).copied().unwrap_or(0);
            
            if load > target_load + 1 {
                // Need to move shards off this node
            }
        }
    }
      }
