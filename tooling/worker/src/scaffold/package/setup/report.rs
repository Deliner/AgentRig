use super::{Config, Files, reconcile::Installation};
use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::{collections::BTreeSet, fs, path::Path};

pub fn configuration(root: &Path, config: &Config, files: &Files) -> Result<Value> {
    let receipt = super::source(root, files, &config.paths.service_path("composition.json"))?;
    let mut configurations = serde_json::Map::new();
    if config.capabilities.lint {
        document(root, files, &mut configurations, &config.paths.lint)?;
    }
    if let Some(delegation) = &config.capabilities.delegation {
        document(root, files, &mut configurations, &delegation.config)?;
    }
    if let Some(review) = &config.capabilities.review {
        let source = super::source(root, files, &review.config)?;
        let settings: review_runner::config::Config = review_runner::config::yaml::decode(&source)?;
        document(root, files, &mut configurations, &review.config)?;
        let location = root.join(&review.config);
        let parent = location
            .parent()
            .context("review configuration directory required")?;
        for tool in settings.tools.values() {
            let path = crate::util::resolve(&parent.join(&tool.project_config))?;
            let relative = path
                .strip_prefix(root)?
                .to_str()
                .context("UTF-8 configuration path required")?;
            document(root, files, &mut configurations, relative)?;
        }
    }
    Ok(json!({
        "schema_version": 1,
        "root": root,
        "configuration": config,
        "configurations": configurations,
        "composition": serde_json::from_str::<Value>(&receipt)?,
    }))
}

fn document(
    root: &Path,
    files: &Files,
    configurations: &mut serde_json::Map<String, Value>,
    path: &str,
) -> Result<()> {
    let source = super::source(root, files, path)?;
    configurations.insert(path.into(), review_runner::config::yaml::decode(&source)?);
    Ok(())
}

pub fn prepared(root: &Path, config: &Config, installation: &Installation) -> Result<Value> {
    let hooks = config.paths.service_path("hooks");
    let repository = config.vcs.backend.repository(root)?;
    let registered = match &repository {
        Some(repository) => repository.hooks_registered(&hooks)?,
        None => false,
    };
    let registration =
        review_runner::vcs::Repository::new(root, config.vcs.backend).expected_registration(&hooks);
    Ok(json!({
        "preview": true,
        "root": root,
        "files": installation.changes(),
        "registrations": {
            "vcs": {
                "backend": config.vcs.backend,
                "initialize": repository.is_none(),
                "initial_branch": config.vcs.base,
                "hooks_path": hooks,
                "registration": registration,
                "update_registration": !registered,
                "runtime_ignore": (config.vcs.backend == review_runner::vcs::Kind::Mercurial).then(|| format!("{hooks}.hgignore")),
            },
            "codex": codex(&installation.files, config)?,
        },
        "directories": directories(root, config, &installation.files)?,
        "dependencies": dependencies(config),
        "dependency_validation": "doctor runs after installation; preview does not execute dependency probes",
    }))
}

fn codex(files: &Files, config: &Config) -> Result<Value> {
    let settings: toml::Value = toml::from_str(std::str::from_utf8(&files[".codex/config.toml"])?)?;
    let mut servers = serde_json::Map::new();
    for (enabled, name) in [
        (config.capabilities.review.is_some(), "worker_review"),
        (
            config.capabilities.delegation.is_some(),
            "worker_delegation",
        ),
    ]
    .into_iter()
    .chain(
        config
            .environment
            .mcp_servers
            .keys()
            .map(|name| (true, name.as_str())),
    ) {
        let server = settings
            .get("mcp_servers")
            .and_then(|servers| servers.get(name));
        if let Some(server) = server.filter(|_| enabled) {
            let mut managed = serde_json::Map::new();
            for field in ["enabled", "command", "args", "tool_timeout_sec", "env_vars"] {
                if let Some(value) = server.get(field) {
                    managed.insert(field.into(), serde_json::to_value(value)?);
                }
            }
            servers.insert(name.into(), Value::Object(managed));
        }
    }
    Ok(json!({"hooks": ".codex/hooks.json", "mcp_servers": servers}))
}

fn directories(root: &Path, config: &Config, files: &Files) -> Result<Value> {
    let mut paths = vec![config.paths.runtime.clone()];
    if let Some(review) = &config.capabilities.review {
        let bytes = match files.get(&review.config) {
            Some(bytes) => bytes.clone(),
            None => fs::read(root.join(&review.config))?,
        };
        let review_settings: review_runner::config::Config =
            review_runner::config::yaml::decode(std::str::from_utf8(&bytes)?)?;
        let location = super::config::relative(root, &review.config)?;
        let parent = location
            .parent()
            .context("review configuration parent required")?;
        for path in [
            &review_settings.runner.runtime_root,
            &review_settings.runner.report_root,
        ] {
            paths.push(parent.join(path).to_string_lossy().into_owned());
        }
    }
    Ok(json!(paths))
}

fn dependencies(config: &Config) -> Value {
    let mut executables = BTreeSet::from([config.vcs.backend.executable()]);
    // Delegated code results currently use Git internally to produce binary patches.
    let code_delegation = config.capabilities.delegation.is_some();
    if code_delegation {
        executables.insert("git");
    }
    executables.extend(
        config
            .commands
            .values()
            .filter_map(|command| command.argv.first().map(String::as_str)),
    );
    let review = config.capabilities.review.is_some();
    let delegation = config.capabilities.delegation.is_some();
    let isolated = review
        || delegation
        || config.vcs.backend == review_runner::vcs::Kind::Mercurial
        || config.commands.values().any(|command| command.read_only);
    if isolated {
        executables.insert("bwrap");
    }
    let mut frontends = Vec::new();
    for (enabled, variable) in [
        (review, "REVIEW_CODEX_BIN"),
        (delegation, "DELEGATE_CODEX_BIN"),
    ] {
        if enabled {
            frontends.push(json!({"frontend": "native Codex with codex-code-mode-host", "override_env": variable}));
        }
    }
    json!({
        "executables": executables,
        "model_frontends": frontends,
        "systemd_user_scope": delegation || config.processes.foreground == super::config::Containment::Systemd,
    })
}
