use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};
use toml_edit::{DocumentMut, Item, Value};

pub const CODEX: &str = ".codex/config.toml";

pub fn codex(root: &Path) -> Result<Option<Vec<u8>>> {
    let path = root.join(CODEX);
    let absent = !path.is_file();
    if absent {
        return Ok(None);
    }
    let mut document: DocumentMut = fs::read_to_string(path)?.parse()?;
    let mut changed = false;
    for (name, command) in [
        ("worker_review", "review"),
        ("worker_delegation", "delegate"),
    ] {
        let entry = document
            .get_mut("mcp_servers")
            .and_then(|servers| servers.get_mut(name));
        if let Some(entry) = entry {
            changed |= migrate_command(entry, command)?;
        }
    }
    Ok(changed.then(|| document.to_string().into_bytes()))
}

fn migrate_command(entry: &mut Item, command: &str) -> Result<bool> {
    let shell = entry.get("command").and_then(Item::as_str) == Some("sh");
    let Some(args) = entry.get_mut("args").and_then(Item::as_array_mut) else {
        return Ok(false);
    };
    let source = args.get(1).and_then(Value::as_str).unwrap_or("");
    let old = format!(
        "root=$(git rev-parse --show-toplevel) && exec \"$root/.worker/bin/discipline-worker\" {command} mcp --root \"$root\""
    );
    let managed = shell
        && args.len() == 2
        && args.get(0).and_then(Value::as_str) == Some("-c")
        && source == old;
    ensure!(
        managed || !source.contains(".worker/bin/discipline-worker"),
        "custom {command} MCP command references the retired executable; update that command explicitly before migration"
    );
    if managed {
        let replacement = old.replace(".worker/bin/discipline-worker", ".worker/bin/agentrig");
        *args.get_mut(1).context("managed MCP argument required")? = Value::from(replacement);
    }
    Ok(managed)
}
