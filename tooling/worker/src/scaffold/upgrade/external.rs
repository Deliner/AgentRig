use anyhow::{Context, Result, ensure};
use std::{fs, path::Path};
use toml_edit::{DocumentMut, Item, Value};

pub const CODEX: &str = ".codex/config.toml";
const GIT_HOOKS: [&str; 2] = [
    ".worker/hooks/pre-commit",
    ".worker/hooks/reference-transaction",
];
const LEGACY_CALL: &str = "exec \"$root/.worker/bin/discipline-worker\"";

pub fn git_hook(root: &Path, path: &str) -> Result<Option<Vec<u8>>> {
    let unselected = !GIT_HOOKS.contains(&path) || !root.join(path).is_file();
    if unselected {
        return Ok(None);
    }
    let source = fs::read_to_string(root.join(path))?;
    let known = source.lines().any(legacy_call);
    if known {
        let migrated: String = source
            .split_inclusive('\n')
            .map(|line| {
                let legacy = legacy_call(line);
                if legacy {
                    line.replacen(LEGACY_CALL, "exec \"$root/.worker/bin/agentrig\"", 1)
                } else {
                    line.to_owned()
                }
            })
            .collect();
        return Ok(Some(migrated.into_bytes()));
    }
    Ok(None)
}

pub fn validate_git_hooks(plan: &super::model::Plan, directory: &Path) -> Result<()> {
    for path in GIT_HOOKS {
        let hash = plan
            .files
            .get(path)
            .and_then(|change| change.after.sha256.as_ref());
        if let Some(hash) = hash {
            let bytes = super::storage::payload(directory, hash)?;
            ensure!(
                !String::from_utf8_lossy(&bytes).lines().any(legacy_call),
                "{path} still references the retired executable; choose the migrated replacement or update the custom adapter and prepare a new plan"
            );
        }
    }
    Ok(())
}

fn legacy_call(line: &str) -> bool {
    line.trim_start().starts_with(LEGACY_CALL)
}

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
        let replacement = crate::scaffold::package::adapters::mcp_command(".worker", command);
        *args.get_mut(1).context("managed MCP argument required")? = Value::from(replacement);
    }
    Ok(managed)
}
