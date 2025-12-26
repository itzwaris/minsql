use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub node_id: u32,
    pub data_dir: String,
    pub port: u16,
    pub peers: Vec<String>,
    pub buffer_pool_size: usize,
    pub wal_buffer_size: usize,
    pub deterministic: bool,
    pub num_shards: usize,
}

impl Config {
    pub fn from_args() -> Result<Self> {
        let args: Vec<String> = env::args().collect();

        let mut node_id = 1;
        let mut data_dir = "./data".to_string();
        let mut port = 5433;
        let mut peers = Vec::new();

        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--node-id" => {
                    node_id = args[i + 1].parse().context("Invalid node-id")?;
                    i += 2;
                }
                "--data-dir" => {
                    data_dir = args[i + 1].clone();
                    i += 2;
                }
                "--port" => {
                    port = args[i + 1].parse().context("Invalid port")?;
                    i += 2;
                }
                "--peers" => {
                    peers = args[i + 1].split(',').map(|s| s.to_string()).collect();
                    i += 2;
                }
                _ => {
                    i += 1;
                }
            }
        }

        Ok(Config {
            node_id,
            data_dir,
            port,
            peers,
            buffer_pool_size: 1024,
            wal_buffer_size: 65536,
            deterministic: false,
            num_shards: 16,
        })
    }

    pub fn data_path(&self) -> PathBuf {
        PathBuf::from(&self.data_dir)
    }
}
