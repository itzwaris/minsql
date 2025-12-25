use anyhow::{Context, Result};
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    Query = 1,
    QueryResponse = 2,
    Error = 3,
    Execute = 4,
    ExecuteResponse = 5,
}

impl MessageType {
    pub fn from_u8(val: u8) -> Result<Self> {
        match val {
            1 => Ok(MessageType::Query),
            2 => Ok(MessageType::QueryResponse),
            3 => Ok(MessageType::Error),
            4 => Ok(MessageType::Execute),
            5 => Ok(MessageType::ExecuteResponse),
            _ => anyhow::bail!("Unknown message type: {}", val),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            MessageType::Query => 1,
            MessageType::QueryResponse => 2,
            MessageType::Error => 3,
            MessageType::Execute => 4,
            MessageType::ExecuteResponse => 5,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub message_type: MessageType,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(message_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            message_type,
            payload,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u32((self.payload.len() + 1) as u32);
        buf.put_u8(self.message_type.to_u8());
        buf.put_slice(&self.payload);
        buf.to_vec()
    }

    pub async fn read_from(stream: &mut TcpStream) -> Result<Self> {
        let length = stream.read_u32().await.context("Failed to read frame length")?;
        
        if length == 0 || length > 100 * 1024 * 1024 {
            anyhow::bail!("Invalid frame length: {}", length);
        }

        let message_type_byte = stream.read_u8().await.context("Failed to read message type")?;
        let message_type = MessageType::from_u8(message_type_byte)?;

        let payload_len = (length - 1) as usize;
        let mut payload = vec![0u8; payload_len];
        stream.read_exact(&mut payload).await.context("Failed to read payload")?;

        Ok(Self {
            message_type,
            payload,
        })
    }

    pub async fn write_to(&self, stream: &mut TcpStream) -> Result<()> {
        let encoded = self.encode();
        stream.write_all(&encoded).await?;
        Ok(())
    }
}
