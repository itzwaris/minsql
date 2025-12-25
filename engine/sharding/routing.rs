use crate::sharding::keyspace::{Keyspace, ShardId};
use crate::language::intent::Intent;
use std::collections::HashMap;

pub struct Router {
    keyspace: Keyspace,
    shard_map: HashMap<ShardId, ShardInfo>,
}

#[derive(Debug, Clone)]
pub struct ShardInfo {
    pub shard_id: ShardId,
    pub node_id: u32,
    pub is_primary: bool,
}

impl Router {
    pub fn new(num_shards: usize) -> Self {
        let keyspace = Keyspace::new(num_shards);
        let mut shard_map = HashMap::new();

        for i in 0..num_shards {
            shard_map.insert(
                ShardId(i as u32),
                ShardInfo {
                    shard_id: ShardId(i as u32),
                    node_id: (i % 3) as u32,
                    is_primary: true,
                },
            );
        }

        Self {
            keyspace,
            shard_map,
        }
    }

    pub fn route(&self, _intent: &Intent) -> Vec<ShardId> {
        self.shard_map.keys().copied().collect()
    }

    pub fn route_key(&self, key: &[u8]) -> ShardId {
        self.keyspace.lookup(key)
    }

    pub fn get_shard_info(&self, shard_id: ShardId) -> Option<&ShardInfo> {
        self.shard_map.get(&shard_id)
    }

    pub fn all_shards(&self) -> Vec<ShardId> {
        self.shard_map.keys().copied().collect()
    }
}
