use anyhow::{Context, Result};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::shared::protocol::HostMessage;

/// Read one Native Messaging frame from an async reader.
/// Format: 4-byte little-endian length prefix, then that many bytes of UTF-8 JSON.
pub async fn read_message<R: AsyncReadExt + Unpin>(reader: &mut R) -> Result<Option<HostMessage>> {
    let mut len_buf = [0u8; 4];
    match reader.read_exact(&mut len_buf).await {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(e) => return Err(e).context("failed to read message length"),
    }
    let len = u32::from_le_bytes(len_buf) as usize;

    if len == 0 {
        return Ok(None);
    }

    // Chrome caps native messages at 1 MB
    anyhow::ensure!(len <= 1_048_576, "native message too large: {len} bytes");

    let mut buf = vec![0u8; len];
    reader
        .read_exact(&mut buf)
        .await
        .context("failed to read message body")?;

    let msg: HostMessage =
        serde_json::from_slice(&buf).context("failed to parse native message JSON")?;
    Ok(Some(msg))
}

/// Write one Native Messaging frame to an async writer.
pub async fn write_message<W: AsyncWriteExt + Unpin>(
    writer: &mut W,
    msg: &HostMessage,
) -> Result<()> {
    let json = serde_json::to_vec(msg).context("failed to serialize message")?;
    let len = json.len() as u32;
    writer
        .write_all(&len.to_le_bytes())
        .await
        .context("failed to write message length")?;
    writer
        .write_all(&json)
        .await
        .context("failed to write message body")?;
    writer.flush().await.context("failed to flush")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::protocol::{HelloMessage, RequestMessage};

    #[tokio::test]
    async fn round_trip_hello() {
        let msg = HostMessage::Hello(HelloMessage {
            instance_id: "test-id".into(),
            browser: "chrome".into(),
            extension_id: "ext".into(),
            version: "1.0".into(),
            capabilities: vec![],
        });

        let mut buf = Vec::new();
        write_message(&mut buf, &msg).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let parsed = read_message(&mut cursor).await.unwrap().unwrap();

        match parsed {
            HostMessage::Hello(h) => assert_eq!(h.instance_id, "test-id"),
            _ => panic!("expected Hello"),
        }
    }

    #[tokio::test]
    async fn eof_returns_none() {
        let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
        let result = read_message(&mut cursor).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn round_trip_request() {
        let msg = HostMessage::Request(RequestMessage {
            id: "req-1".into(),
            method: "getSiteSearchResult".into(),
            params: serde_json::json!({"siteId": "test"}),
        });

        let mut buf = Vec::new();
        write_message(&mut buf, &msg).await.unwrap();

        let mut cursor = std::io::Cursor::new(buf);
        let parsed = read_message(&mut cursor).await.unwrap().unwrap();

        match parsed {
            HostMessage::Request(r) => {
                assert_eq!(r.id, "req-1");
                assert_eq!(r.method, "getSiteSearchResult");
            }
            _ => panic!("expected Request"),
        }
    }
}
