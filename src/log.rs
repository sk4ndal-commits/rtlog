//! Log ingestion layer: defines a generic interface for streaming log lines from sources.
//! 
//! This module follows SOLID principles by introducing an abstraction (`LogSource`) that can be
//! implemented by different backends (files, sockets, etc.). The application runtime depends on
//! this interface instead of a concrete file reader.

use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

/// Generic trait for log sources.
///
/// Implementors should continuously send lines to the provided channel.
#[async_trait::async_trait]
pub trait LogSource {
    async fn stream(self, source_id: usize, tx: Sender<(usize, String)>) -> Result<()>;
}

/// Concrete file-tail source. If `follow` is true, it behaves like `tail -f`.
pub struct FileTail {
    pub path: PathBuf,
    pub follow: bool,
}

#[async_trait::async_trait]
impl LogSource for FileTail {
    async fn stream(self, source_id: usize, tx: Sender<(usize, String)>) -> Result<()> {
        let mut file = File::open(&self.path).await?;
        if self.follow {
            file.seek(SeekFrom::End(0)).await?;
        }
        let mut reader = BufReader::new(file);
        let mut buf = String::new();
        loop {
            buf.clear();
            match reader.read_line(&mut buf).await? {
                0 => {
                    if self.follow {
                        sleep(Duration::from_millis(200)).await;
                        continue;
                    } else {
                        break; // EOF and not following
                    }
                }
                _ => {
                    if buf.ends_with('\n') { buf.pop(); }
                    if buf.ends_with('\r') { buf.pop(); }
                    if tx.send((source_id, buf.clone())).await.is_err() {
                        break; // receiver gone
                    }
                }
            }
        }
        Ok(())
    }
}

/// Backwards-compatible helper that streams a file using the new `FileTail` implementor.
pub async fn stream_file(path: PathBuf, follow: bool, source_id: usize, tx: Sender<(usize, String)>) -> Result<()> {
    FileTail { path, follow }.stream(source_id, tx).await
}
