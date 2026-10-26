# State

## Focus

Record the user's remembered target state after the loss of about 1500
commits made on another machine. The user dictates it in parts: functionality,
code structure, then what to do. Parts 1 and 2 are recorded in
[Requests.md](Requests.md) under "Целевое состояние после утраты истории —
2026-10-26"; the user will add more. origin has no commits newer than master.
No implementation is authorized yet.

## Workspace

Branch: feature/target-state

Revision: a14f284660689fd6d667a276710875a30dd673db

The branch was created by feature-start from master at the same revision.
The uncommitted VAC changes only Requests.md and this State. The machine is
macOS (Darwin 25.5.0). The pinned runtime 9aee4ae was bootstrapped there with
a canonical TMPDIR, because the /var symlink breaks init path checks.
A stray branch feature/init-probe at a14f284 remains from an install probe;
guard-reference refuses its deletion.

## Progress

Parts 1 and 2 are recorded as the user's own requirements, not as agreed
Backlog cards. The new direction removes review, delegation, Mercurial,
upgrade and JS/TS/TSX, moves configuration and just recipes into modular
directories, moves the runtime code into Project for dogfooding, limits
Markdown size, and makes the linter the main product. The nested-feature
code structure (features/, single-file features, shared/) is remembered
only partly and must be reconstructed.
It is not yet reconciled with the pending Plan P018–P023 or with existing
decisions; that happens through Backlog and Decisions after all three parts.

## Verification

git diff shows changes only in the new Requests.md section and this State.
memory-check, catalog commands and the staged gate do not run on macOS:
job identity reads /proc, and the configured foreground mode needs systemd.

## Blockers

The staged gate cannot run on macOS. On 2026-10-26 the user authorized
committing these Requests.md and State records without the pre-commit hook;
this does not extend to code changes. feature-merge still needs the full gate.

## Next action

Receive the remaining parts from the user, including the Ledger rework, and
append them to Requests.md. Then draft Backlog cards in chat for agreement.
