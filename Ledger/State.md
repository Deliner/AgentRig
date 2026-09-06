# State

## Focus

Deliver P005: independent stable development environment and public GitHub/MIT distribution, applying complexity-discipline.

## Workspace

Branch: feature/stable-distribution

Revision: e1af2ff

Started from clean master after observing the completed P004 merge. No merge/rebase or upgrade is active.

## Progress

Stable-runtime separation and distribution are committed. Public repository https://github.com/Deliner/AgentRig exists, GitHub recognizes MIT, and origin tracks the published feature/stable-distribution branch at e1af2ff. A fresh clone independently installed the pinned runtime, registered .githooks and remained clean. First GitHub CI exposed the absent native Codex/code-mode-host dependency for setup tests. Current VAC installs the pinned official package in CI, preserving all checks. Release publication and local integration remain pending.

## Verification

Full staged gates at e986ec2 and e1af2ff passed 363 Python and 50 Rust tests and all other checks. Independent consumer and clean-clone receipts are .tmp/publication-tools/release-acceptance.json and clone-acceptance.json. GitHub run 34048048545 passed build/lint/memory/formatting/static checks and Rust unit tests, then failed 25 setup/migration Python cases because native Codex was not installed; 338 Python cases passed. The official 0.153.4 package layout and published SHA-256 were inspected for the CI fix. Historical Gitleaks review found only private-key detection string literals. Remote acceptance remains pending.

## Blockers

The downloaded official Codex package passed its published checksum and version check; all 27 setup tests pass with it explicitly selected. Updated workflow passes actionlint.

None observed.

## Next action

Verify and commit the CI dependency fix through the staged gate, push the feature branch, then observe the new GitHub CI run. Verify its downloadable archive and publish the versioned release. Record acceptance, integrate P005 on master, retain the feature branch, push integration and select master as GitHub's default branch. Reconcile live GitHub/Git state before repeating publication operations.
