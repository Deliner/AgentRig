use anyhow::{Result, bail, ensure};
use serde_json::{Map, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

#[derive(Default)]
pub struct Values {
    pub configuration: Map<String, Value>,
    pub provenance: BTreeMap<String, PathBuf>,
}

struct Layer<'a> {
    source: &'a Path,
    overrides: BTreeSet<String>,
    provenance: &'a mut BTreeMap<String, PathBuf>,
}

impl Values {
    pub fn add_at(
        &mut self,
        source: &Path,
        values: Map<String, Value>,
        overrides: &[String],
        target: &str,
    ) -> Result<()> {
        let mut wrapped = values;
        let scoped = !target.is_empty();
        if scoped {
            ensure!(
                target.starts_with('/'),
                "package into must be a JSON pointer"
            );
            for part in target[1..].split('/').rev() {
                let key = part.replace("~1", "/").replace("~0", "~");
                ensure!(
                    !key.is_empty() && address("", &key) == format!("/{part}"),
                    "package into must contain nonempty mapping keys with valid JSON pointer escapes"
                );
                wrapped = Map::from_iter([(key, Value::Object(wrapped))]);
            }
        }
        let overrides: Vec<_> = overrides
            .iter()
            .map(|path| format!("{target}{path}"))
            .collect();
        self.add(source, wrapped, &overrides)
    }

    pub fn add(
        &mut self,
        source: &Path,
        values: Map<String, Value>,
        overrides: &[String],
    ) -> Result<()> {
        let selected: BTreeSet<_> = overrides.iter().cloned().collect();
        ensure!(
            selected.len() == overrides.len(),
            "duplicate override in {}",
            source.display()
        );
        let mut layer = Layer {
            source,
            overrides: selected,
            provenance: &mut self.provenance,
        };
        layer.mapping(&mut self.configuration, values, "")?;
        ensure!(
            layer.overrides.is_empty(),
            "unused override in {}: {:?}; overrides must replace an existing value",
            source.display(),
            layer.overrides
        );
        Ok(())
    }
}

impl Layer<'_> {
    fn mapping(
        &mut self,
        target: &mut Map<String, Value>,
        source: Map<String, Value>,
        path: &str,
    ) -> Result<()> {
        for (key, value) in source {
            let address = address(path, &key);
            match target.get_mut(&key) {
                Some(current) => self.merge(current, value, &address)?,
                None => {
                    self.record(&value, &address)?;
                    target.insert(key, value);
                }
            }
        }
        Ok(())
    }

    fn merge(&mut self, target: &mut Value, source: Value, path: &str) -> Result<()> {
        let replacing = self.overrides.remove(path);
        if replacing {
            let prefix = format!("{path}/");
            self.provenance
                .retain(|key, _| key != path && !key.starts_with(&prefix));
            self.record(&source, path)?;
            *target = source;
            return Ok(());
        }
        match (target, source) {
            (Value::Object(current), Value::Object(incoming)) => {
                self.provenance.remove(path);
                self.mapping(current, incoming, path)
            }
            (Value::Array(current), Value::Array(incoming)) if named_list(path) => {
                self.provenance.remove(path);
                self.sequence(current, incoming, path)
            }
            _ => bail!(
                "configuration conflict at {path}: {} and {}; add an explicit override",
                self.provenance.get(path).map_or_else(
                    || "earlier package".into(),
                    |origin| origin.display().to_string()
                ),
                self.source.display()
            ),
        }
    }

    fn sequence(&mut self, target: &mut Vec<Value>, source: Vec<Value>, path: &str) -> Result<()> {
        let mut seen = BTreeSet::new();
        for value in source {
            let id = identity(&value)?;
            ensure!(seen.insert(id.to_owned()), "duplicate ID {id} at {path}");
            let address = address(path, id);
            let current = target
                .iter_mut()
                .find(|item| item.get("id") == value.get("id"));
            match current {
                Some(current) => {
                    ensure!(
                        self.overrides.contains(&address),
                        "configuration conflict at {address}; named entries require an explicit override"
                    );
                    self.merge(current, value, &address)?;
                }
                None => {
                    self.record(&value, &address)?;
                    target.push(value);
                }
            }
        }
        Ok(())
    }

    fn record(&mut self, value: &Value, path: &str) -> Result<()> {
        self.provenance.insert(path.into(), self.source.into());
        match value {
            Value::Object(values) => {
                for (key, value) in values {
                    self.record(value, &address(path, key))?;
                }
            }
            Value::Array(values) if named_list(path) => {
                let mut seen = BTreeSet::new();
                for value in values {
                    let id = identity(value)?;
                    ensure!(seen.insert(id), "duplicate ID {id} at {path}");
                    self.record(value, &address(path, id))?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

fn named_list(path: &str) -> bool {
    matches!(path, "/checks" | "/rules")
}

fn identity(value: &Value) -> Result<&str> {
    let id = value.get("id").and_then(Value::as_str).unwrap_or("");
    ensure!(
        !id.is_empty(),
        "named configuration entries require a nonempty string id"
    );
    Ok(id)
}

fn address(parent: &str, key: &str) -> String {
    format!("{parent}/{}", key.replace('~', "~0").replace('/', "~1"))
}
