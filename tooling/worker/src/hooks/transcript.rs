use anyhow::Result;
use serde_json::Value;
use std::{
    fs::File,
    io::{BufRead, BufReader, Seek, SeekFrom},
    path::Path,
};

// DECISION: D015
pub fn tokens(path: &Path, mut offset: u64) -> Result<(Vec<u64>, u64)> {
    let file = File::open(path)?;
    if offset > file.metadata()?.len() {
        offset = 0;
    }
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(offset))?;
    let mut tokens = Vec::new();
    loop {
        let mut line = Vec::new();
        let count = reader.read_until(b'\n', &mut line)?;
        if count == 0 {
            break;
        }
        let record: Value = match serde_json::from_slice(&line) {
            Ok(value) => value,
            Err(_) if !line.ends_with(b"\n") => break,
            Err(_) => {
                offset += count as u64;
                continue;
            }
        };
        offset += count as u64;
        if record["type"] == "event_msg"
            && record["payload"]["type"] == "token_count"
            && let Some(value) =
                record["payload"]["info"]["last_token_usage"]["input_tokens"].as_u64()
        {
            tokens.push(value);
        }
    }
    Ok((tokens, offset))
}
