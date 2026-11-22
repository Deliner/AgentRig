---
name: propose-protected-change
description: Propose a change of a protected enforcement file, such as a hook, a launcher, a recipe, a configuration module or the pin, or a new lint exception for the user's approval, since agents never change these files themselves and never lift a rule unseen.
---

# Let the user approve a change of enforcement

Protected files decide what an agent may run and what a commit must pass, so an
agent that changed them could switch the checks off. The agent hook refuses an
edit or a `just write` operand that reaches them, and the commit gate refuses a
staged change of them that the user has not approved. They are:

- the launcher, Git hooks and Claude Code mods in the service directory
  (`bin/`, `hooks/`, `mods/`), the
  agent hook registrations `.claude/settings.json`, `.codex/hooks.json` and
  `.codex/config.toml`, the recipes `justfile` and `.agentrig/just/`;
- the configuration modules `.agentrig/project.yaml`, `git.yaml`,
  `commands.yaml`, `checks.yaml`, `hooks.yaml`, `tools.yaml` and `rust.yaml`,
  the lint configuration `paths.lint`, so a new catalog
  command, `just write` program or check setting is a proposal too;
- `.git`, the route state `reminders/`, the approval record `approved`, the
  consent record `consent` and the compaction requirement `compaction` in the
  runtime directory;
- every path listed in `hooks.protected` and the pin named by `git.pin`.

## Propose

1. Write the complete new content of each changed file to
   `<paths.runtime>/proposal/<path from the project root>`, for example
   `.cache/runtime/proposal/.agentrig/commands.yaml`. An existing file keeps its
   mode; a new file takes the mode of its proposal.
2. Tell the user which files change and why, and ask them to type
   `! just approve` in Claude Code, or to run the same command without `!` in
   their own terminal. Never run it yourself: the hook refuses it, and only the
   user approves.
3. The approve command applies the proposal, undoes it when the configuration
   no longer loads, records the protected content that now differs from the
   last commit, including the user's own edits and deletions, prints it and
   removes the proposal. Stage and commit the approved files; a later change
   needs a new approval.

A deletion has no proposal: ask the user to delete the file before approving.
To move the pin, run the `propose-pin` command with a revision that descends
from the pinned one. It writes the pin's proposal and lists the source files
changed since the pinned revision; show that list when you ask for approval.
The approve command first runs the installer named by `git.install` with the
revision and writes the pin only when the install succeeds; the gate accepts a
changed pin only with approved content that descends from the committed one.
While the user's consent holds, which only the user gives by typing
`! just consent`, the `propose-pin` command takes only a clean `HEAD`, runs the
branch gate, installs the revision and writes the pin as approved content;
commit it with State, and the `consented-pins` command lists those pins.
Never run the installer with a revision: the hook refuses it, and without one
it installs the pinned revision. A later change that needs an earlier one in
the installed runtime waits for the user's approval of a pin mid-branch. When
the configuration is invalid in a protected file, ask the user to correct it.

Edit the lint configuration itself to add a rule: the gate accepts it without
approval while every committed rule stays unchanged and in order and the other
settings stay equal. Removing, disabling or changing a rule, a threshold or an
exclusion is a proposal.

## Lint exceptions

An exception `DECISION: DNNN RULE` above a line lifts a lint rule, so the gate
refuses a commit that adds one the user has not approved. It is new when its
file, decision, rule or the text of the line beneath it differs from every
exception of the last commit; a deletion needs no approval. Show the user the
refused list, each entry naming file:line, decision, rule and the line it
lifts, and ask them to type `! just approve`. The approve command records the
new exceptions of the working tree with any protected change and prints them;
stage and commit them as they are, since a later edit of an exception or its
line needs a new approval.
