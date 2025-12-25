use blake3::Hash;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ShardId(pub u32);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRange {
    pub start: Vec<u8>,
    pub end: Vec<u8>,
    pub shard_id: ShardId,
}

pub struct Keyspace {
    pub ranges: Vec<KeyRange>,
    pub num_shards: usize,
}

impl Keyspace {
    pub fn new(num_shards: usize) -> Self {
        let mut ranges = Vec::new();

        for i in 0..num_shards {
            ranges.push(KeyRange {
                start: vec![],
                end: vec![],
                shard_id: ShardId(i as u32),
            });
        }

        Self { ranges, num_shards }
    }

    pub fn lookup(&self, key: &[u8]) -> ShardId {
        let hash = blake3::hash(key);
        let hash_bytes = hash.as_bytes();
        let hash_u64 = u64::from_le_bytes(hash_bytes[0..8].try_into().unwrap());
        ShardId((hash_u64 % self.num_shards as u64) as u32)
    }

    pub fn get_shard_range(&self, shard_id: ShardId) -> Option<&KeyRange> {
        self.ranges.iter().find(|r| r.shard_id == shard_id)
    }
      }
