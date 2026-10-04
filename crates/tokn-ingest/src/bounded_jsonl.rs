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

pub fn read_jsonl<F>(path: &Path, cfg: &JsonlConfig, sink: F) -> Result<IngestStats, IngestError>
where
    F: FnMut(ParsedRecord),
{
    let file = File::open(path)?;
    let snapshot_end = file.metadata()?.len();
    let mut reader = BufReader::with_capacity(cfg.io_buffer_bytes, file);
    read_jsonl_snapshot(&mut reader, snapshot_end, cfg, sink)
}

fn read_jsonl_snapshot<R, F>(
    reader: &mut R,
    snapshot_end: u64,
    cfg: &JsonlConfig,
    mut sink: F,
) -> Result<IngestStats, IngestError>
where
    R: BufRead,
    F: FnMut(ParsedRecord),
{
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
        let mut eof_before_snapshot = false;

        loop {
            let buf = reader.fill_buf()?;
            if buf.is_empty() {
                eof_before_snapshot = offset < snapshot_end;
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

        if eof_before_snapshot {
            if offset > start {
                stats.records_seen += 1;
            }
            stats.truncated_tail += 1;
            break;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufReader, Cursor};

    #[test]
    fn stops_when_reader_reaches_eof_before_snapshot_end() {
        let bytes = b"{\"type\":\"a\"}\n{\"type\":\"partial\"";
        let snapshot_end = bytes.len() as u64 + 64;
        let mut reader = BufReader::new(Cursor::new(bytes));
        let mut records = 0;

        let stats = read_jsonl_snapshot(&mut reader, snapshot_end, &JsonlConfig::default(), |_| {
            records += 1
        })
        .unwrap();

        assert_eq!(records, 1);
        assert_eq!(stats.records_seen, 2);
        assert_eq!(stats.records_valid, 1);
        assert_eq!(stats.truncated_tail, 1);
        assert_eq!(stats.malformed_records, 0);
    }

    #[test]
    fn stops_when_snapshot_disappears_before_first_read() {
        let mut reader = BufReader::new(Cursor::new(Vec::<u8>::new()));
        let stats = read_jsonl_snapshot(&mut reader, 128, &JsonlConfig::default(), |_| {
            panic!("no record should be emitted")
        })
        .unwrap();

        assert_eq!(stats.records_seen, 0);
        assert_eq!(stats.records_valid, 0);
        assert_eq!(stats.truncated_tail, 1);
    }
}
