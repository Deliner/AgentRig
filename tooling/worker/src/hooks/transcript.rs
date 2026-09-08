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
    let truncated = offset > file.metadata()?.len();
    if truncated {
        offset = 0;
    }
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::Start(offset))?;
    let mut tokens = Vec::new();
    loop {
        let mut line = Vec::new();
        let count = reader.read_until(b'\n', &mut line)?;
        let end_of_file = count == 0;
        if end_of_file {
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
        tokens.extend(observation(&record));
    }
    Ok((tokens, offset))
}

fn observation(record: &Value) -> Option<u64> {
    match record["type"].as_str()? {
        "event_msg" => {
            let token_record = record["payload"]["type"] == "token_count";
            token_record
                .then(|| record["payload"]["info"]["last_token_usage"]["input_tokens"].as_u64())
                .flatten()
        }
        "assistant" => {
            let usage = &record["message"]["usage"];
            let input = usage["input_tokens"].as_u64()?;
            let created = usage["cache_creation_input_tokens"].as_u64().unwrap_or(0);
            let cached = usage["cache_read_input_tokens"].as_u64().unwrap_or(0);
            input.checked_add(created)?.checked_add(cached)
        }
        _ => None,
    }
}
