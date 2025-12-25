use anyhow::{Context, Result};
use bytes::{Buf, BufMut, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAGIC_BYTES: &[u8] = b"MINSQL";

#[derive(Debug, Clone)]
pub struct HandshakeRequest {
    pub protocol_version: u32,
    pub client_name: String,
}

#[derive(Debug, Clone)]
pub struct HandshakeResponse {
    pub protocol_version: u32,
    pub server_version: String,
    pub node_id: u32,
}

impl HandshakeRequest {
    pub fn new(client_name: String) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            client_name,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_slice(MAGIC_BYTES);
        buf.put_u32(self.protocol_version);
        buf.put_u32(self.client_name.len() as u32);
        buf.put_slice(self.client_name.as_bytes());
        buf.to_vec()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        let magic = &buf[..6];
        if magic != MAGIC_BYTES {
            anyhow::bail!("Invalid magic bytes");
        }
        buf.advance(6);

        let protocol_version = buf.get_u32();
        let client_name_len = buf.get_u32() as usize;
        
        if buf.remaining() < client_name_len {
            anyhow::bail!("Incomplete handshake request");
        }

        let client_name = String::from_utf8(buf[..client_name_len].to_vec())
            .context("Invalid client name")?;

        Ok(Self {
            protocol_version,
            client_name,
        })
    }
}

impl HandshakeResponse {
    pub fn new(node_id: u32) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            node_id,
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = BytesMut::new();
        buf.put_u32(self.protocol_version);
        buf.put_u32(self.server_version.len() as u32);
        buf.put_slice(self.server_version.as_bytes());
        buf.put_u32(self.node_id);
        buf.to_vec()
    }

    pub fn decode(data: &[u8]) -> Result<Self> {
        let mut buf = data;

        let protocol_version = buf.get_u32();
        let server_version_len = buf.get_u32() as usize;
        
        if buf.remaining() < server_version_len {
            anyhow::bail!("Incomplete handshake response");
        }

        let server_version = String::from_utf8(buf[..server_version_len].to_vec())
            .context("Invalid server version")?;
        buf.advance(server_version_len);

        let node_id = buf.get_u32();

        Ok(Self {
            protocol_version,
            server_version,
            node_id,
        })
    }
}

pub async fn perform_handshake(stream: &mut TcpStream, node_id: u32) -> Result<HandshakeRequest> {
    let mut buf = vec![0u8; 1024];
    let n = stream.read(&mut buf).await.context("Failed to read handshake")?;
    
    if n == 0 {
        anyhow::bail!("Connection closed during handshake");
    }

    let request = HandshakeRequest::decode(&buf[..n])?;

    if request.protocol_version != PROTOCOL_VERSION {
        anyhow::bail!("Unsupported protocol version: {}", request.protocol_version);
    }

    let response = HandshakeResponse::new(node_id);
    stream.write_all(&response.encode()).await?;

    Ok(request)
}
