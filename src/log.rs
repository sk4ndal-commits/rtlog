use std::path::PathBuf;
use std::time::Duration;

use anyhow::Result;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, AsyncSeekExt, BufReader, SeekFrom};
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

/// Stream lines from a file. If `follow` is true, keep polling for new data.
pub async fn stream_file(path: PathBuf, follow: bool, tx: Sender<String>) -> Result<()> {
    let mut file = File::open(&path).await?;

    // If following, seek to end so we only get new lines; otherwise read from start
    if follow {
        file.seek(SeekFrom::End(0)).await?;
    }

    let mut reader = BufReader::new(file);
    let mut buf = String::new();

    loop {
        buf.clear();
        match reader.read_line(&mut buf).await? {
            0 => {
                if follow {
                    sleep(Duration::from_millis(200)).await;
                    continue;
                } else {
                    break; // EOF and not following
                }
            }
            _n => {
                // Trim only trailing newlines, keep content
                if buf.ends_with('\n') { buf.pop(); }
                if buf.ends_with('\r') { buf.pop(); }
                if tx.send(buf.clone()).await.is_err() {
                    break; // receiver gone
                }
            }
        }
    }

    Ok(())
}
