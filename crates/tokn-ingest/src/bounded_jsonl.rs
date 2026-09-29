use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use thiserror::Error;

use crate::{IngestStats, ParsedRecord};

#[derive(Debug, Clone)]
pub struct JsonlConfig {
    pub io_buffer_bytes: usize,
    pub max_record_bytes: usize,
}

impl Default for JsonlConfig {
    fn default() -> Self {
        Self {
            io_buffer_bytes: 256 * 1024,
            max_record_bytes: 32 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Error)]
pub enum IngestError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub fn read_jsonl<F>(
    path: &Path,
    cfg: &JsonlConfig,
    mut sink: F,
) -> Result<IngestStats, IngestError>
where
    F: FnMut(ParsedRecord),
{
    let file = File::open(path)?;
    let snapshot_end = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(cfg.io_buffer_bytes, file);
    let mut stats = IngestStats {
        snapshot_bytes: snapshot_end,
        ..Default::default()
    };
    let mut snapshot_hasher = blake3::Hasher::new();
    let mut offset = 0_u64;
    let mut line = 1_u64;

    while offset < snapshot_end {
        let start = offset;
        let mut bytes = Vec::new();
        let mut hasher = blake3::Hasher::new();
        let mut oversized = false;
        let mut ended_newline = false;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                break;
            }

            let remaining = (snapshot_end - offset) as usize;
            let view = &buf[..buf.len().min(remaining)];
            let take = view
                .iter()
                .position(|b| *b == b'\n')
                .map(|i| i + 1)
                .unwrap_or(view.len());
            let chunk = &view[..take];

            hasher.update(chunk);
            snapshot_hasher.update(chunk);
            if !oversized && bytes.len() + chunk.len() <= cfg.max_record_bytes {
                bytes.extend_from_slice(chunk);
            } else {
                oversized = true;
                bytes.clear();
            }

            ended_newline = chunk.last() == Some(&b'\n');
            reader.consume(take);
            offset += take as u64;

            if ended_newline || offset >= snapshot_end {
                break;
            }
        }

        stats.records_seen += 1;
        if oversized {
            stats.oversized_records += 1;
            line += 1;
            continue;
        }

        if ended_newline {
            bytes.pop();
        }
        if bytes.last() == Some(&b'\r') {
            bytes.pop();
        }
        let hash = hasher.finalize().to_hex().to_string();

        match serde_json::from_slice(&bytes) {
            Ok(value) => {
                stats.records_valid += 1;
                sink(ParsedRecord {
                    offset: start,
                    line,
                    byte_length: offset - start,
                    hash,
                    value,
                });
            }
            Err(_) if !ended_newline && offset >= snapshot_end => stats.truncated_tail += 1,
            Err(_) => stats.malformed_records += 1,
        }

        line += 1;
    }

    stats.snapshot_hash = snapshot_hasher.finalize().to_hex().to_string();
    Ok(stats)
}
