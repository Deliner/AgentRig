// DECISION: D179
use crate::shared::console;
use crate::shared::vcs::{Repository, Settings};
use anyhow::Result;

#[derive(Clone, Copy)]
enum Replacement {
    FastForward,
    Lease,
}

struct Push<'a> {
    remote: &'a str,
    branch: &'a str,
    replacement: Replacement,
}

impl Push<'_> {
    fn arguments(&self) -> Vec<String> {
        let reference = format!("refs/heads/{}", self.branch);
        let lease = match self.replacement {
            Replacement::FastForward => None,
            Replacement::Lease => Some(format!("--force-with-lease={reference}")),
        };
        let target = [self.remote.to_owned(), format!("{reference}:{reference}")];
        ["push".to_owned()]
            .into_iter()
            .chain(lease)
            .chain(target)
            .collect()
    }

    fn run(&self, repository: &Repository<'_>) -> Result<(), String> {
        let arguments = self.arguments();
        let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
        repository.run(&arguments).map(drop).map_err(|error| {
            format!(
                "{branch} not pushed to {remote}: {error:#}\nretry: git {retry}",
                branch = self.branch,
                remote = self.remote,
                retry = shell_words::join(&arguments)
            )
        })
    }
}

pub(super) fn merged(repository: &Repository<'_>, settings: &Settings, feature: &str) -> i32 {
    let Some(remote) = settings.remote.as_deref() else {
        return 0;
    };
    let base = Push {
        remote,
        branch: &settings.base,
        replacement: Replacement::FastForward,
    };
    let feature = Push {
        remote,
        branch: feature,
        replacement: Replacement::Lease,
    };
    let mut pushed = Vec::new();
    let mut failed = false;
    for push in [base, feature] {
        match push.run(repository) {
            Ok(()) => pushed.push(push.branch),
            Err(failure) => {
                failed = true;
                console::stderr(format!("merge kept; {failure}"));
            }
        }
    }
    let reported = !pushed.is_empty();
    if reported {
        console::stdout(format!("pushed {} to {remote}", pushed.join(", ")));
    }
    i32::from(failed)
}

pub(super) fn committed(repository: &Repository<'_>, settings: &Settings) -> Result<i32> {
    let Some(remote) = settings.remote.as_deref() else {
        return Ok(0);
    };
    let (branch, _) = repository.commit_context()?;
    let unprefixed = !branch.starts_with(&settings.prefix);
    if unprefixed {
        return Ok(0);
    }
    let push = Push {
        remote,
        branch: &branch,
        replacement: Replacement::Lease,
    };
    let Err(failure) = push.run(repository) else {
        console::stdout(format!("pushed {branch} to {remote}"));
        return Ok(0);
    };
    console::stderr(format!(
        "WARNING: commit kept; {failure}\nthe next commit or feature-merge pushes it again"
    ));
    Ok(1)
}
