use crate::sharding::keyspace::ShardId;
use anyhow::Result;

pub struct Rebalancer;

impl Rebalancer {
    pub fn new() -> Self {
        Self
    }

    pub fn split_shard(&self, shard_id: ShardId) -> Result<(ShardId, ShardId)> {
        let new_shard_a = ShardId(shard_id.0 * 2);
        let new_shard_b = ShardId(shard_id.0 * 2 + 1);

        Ok((new_shard_a, new_shard_b))
    }

    pub fn migrate_shard(&self, shard_id: ShardId, from_node: u32, to_node: u32) -> Result<()> {
        tracing::info!(
            "Migrating shard {:?} from node {} to node {}",
            shard_id,
            from_node,
            to_node
        );
        Ok(())
    }

    pub fn should_split(&self, shard_size: usize, threshold: usize) -> bool {
        shard_size > threshold
    }
}
