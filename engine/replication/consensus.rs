use crate::ffi::storage::StorageEngine;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub struct RaftNode {
    node_id: u32,
    peers: Vec<String>,
    storage: Arc<StorageEngine>,
    command_tx: mpsc::Sender<Vec<u8>>,
    command_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Vec<u8>>>>,
}

impl RaftNode {
    pub fn new(node_id: u32, peers: Vec<String>, storage: Arc<StorageEngine>) -> Result<Self> {
        let (command_tx, command_rx) = mpsc::channel(1000);

        Ok(Self {
            node_id,
            peers,
            storage,
            command_tx,
            command_rx: Arc::new(tokio::sync::Mutex::new(command_rx)),
        })
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    pub async fn propose_command(&self, command: Vec<u8>) -> Result<()> {
        self.command_tx.send(command).await?;
        Ok(())
    }

    pub async fn run(self: Arc<Self>) -> Result<()> {
        loop {
            let mut rx = self.command_rx.lock().await;

            if let Some(_command) = rx.recv().await {
                drop(rx);
            } else {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(())
    }
}
