# State

## Focus

Deliver P007, then P008 under the user's active implementation goal; P006 is integrated. Current VAC: select Git/Mercurial for delegated revision inputs and identify the source VCS in retained results.

## Workspace

Branch: feature/vcs-backends

Revision: 8a868c0

P006 is integrated at 0c9cc39. Native VCS reads and selectable review snapshots are committed at 134cf94 and 8a868c0; the latter passed its full gate with 430 Python and 107 Rust tests. The installed development pin is unchanged. Ledger/Requests.md separately preserves the user's requested reminder and original description; it is untracked and must be preserved.

## Progress

P007 is active. Mercurial 7.2.4 is supplied by the locked uv test environment. Review project YAML selects repository.vcs. Delegation YAML now selects top-level vcs, defaulting to Git, for all profiles' source snapshots. Retained input manifests and code reports identify the source VCS. Real Mercurial read/artifact/code runs pass with test clients, including failed checks and patch import into a separate base checkout. The isolated code patch builder still uses Git internally, explicitly documented. Shared delivery configuration, setup, private adapters and delivery/recovery operations remain required. P008 is pending.

## Verification

Forty-six focused Python tests pass across new Mercurial delegation, configuration validation and existing Git code delegation. Four Rust task tests pass. New scenarios verify committed inputs despite dirty source files, read-only mounts, retained VCS/revision identity, checked code success/failure, cleanup and importing the retained patch into a separate Mercurial base checkout. Structural lint has no blocking findings. No real model acceptance is claimed. This VAC's full staged gate is pending; the prior component's passing gate does not prove this content.

## Blockers

None observed. Only Git was initially on PATH; the locked uv environment now provides hg and the review-test catalog command uses it. No corporate VCS access is needed for the authorized private extension example.

## Next action

Commit delegation backend selection through the staged gate. Continue shared project configuration, setup, private adapters and all configured delivery/recovery operations with actual backend capabilities, consolidating remaining VCS-specific code at the shared boundary. Commit the separately requested reminder without losing it. Verify full independent Git/Mercurial consumer acceptance and private extension behavior before P007 integration, then deliver P008. Preserve the installed development pin while testing candidates.
