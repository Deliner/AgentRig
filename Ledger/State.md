# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current maintenance VAC: retain the user's requested remarks and reconcile recovery facts after delegation delivery.

## Workspace

Branch: feature/vcs-backends

Revision: ad0a052

P006 is integrated at 0c9cc39. Native VCS reads, selectable review snapshots and delegation are committed on feature/vcs-backends; ad0a052 passed the full gate with 435 Python and 107 Rust tests. The installed development pin is unchanged. Ledger/Requests.md preserves the user's requested reminder and original description and is included in this maintenance VAC.

## Progress

P007 is active. Mercurial 7.2.4 is supplied by the locked uv test environment. Review project YAML selects repository.vcs. Delegation YAML now selects top-level vcs, defaulting to Git, for all profiles' source snapshots. Retained input manifests and code reports identify the source VCS. Real Mercurial read/artifact/code runs pass with test clients, including failed checks and patch import into a separate base checkout. The isolated code patch builder still uses Git internally, explicitly documented. Shared delivery configuration, setup, private adapters and delivery/recovery operations remain required. P008 is pending.

## Verification

The delegation commit ad0a052 passed all configured gates: 435 Python and 107 Rust tests, formatting, lint, memory checks, Clippy and configured static checks. Focused scenarios verify committed inputs despite dirty source files, read-only mounts, retained VCS/revision identity, checked code success/failure, cleanup and importing the retained patch into a separate Mercurial base checkout. No real model acceptance is claimed. Only the requested remarks and recovery documentation change in the current maintenance VAC; its commit gate is pending.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit the requested remarks and recovery documentation through the staged gate. Continue shared project configuration, setup, private adapters and all configured delivery/recovery operations with actual backend capabilities, consolidating remaining VCS-specific code at the shared boundary. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
