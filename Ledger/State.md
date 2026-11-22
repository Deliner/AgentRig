# State

## Focus

Maintenance under the user's overnight speed-up instruction, no product feature: compute check-evidence SHA-256
fingerprints with the aarch64 hardware instructions by moving sha2 from 0.10 to 0.11, with identical digests.

## Workspace

Branch: feature/hardware-sha256
Revision: 1299a250682ad62ad24af31308e3251680efe2c3

## Progress

Project/Cargo.toml asks for sha2 0.11 without default features; Project/Cargo.lock locks sha2 0.11.1, digest 0.11.3,
block-buffer 0.12.1, crypto-common 0.2.2, hybrid-array 0.4.15 and cpufeatures 0.3.1, and drops generic-array and
version_check. The 0.11 output has no LowerHex, so shared/atomic_files offers one hex helper that both digests use.

## Verification

`just candidate --version` builds with --locked. shared::atomic_files:: (including the known "abc" SHA-256 vector) and
project_memory::check_evidence:: tests pass. A temporary uncommitted probe in an init project measured
`check --only tests` at about 85 ms with the 11.6 MB copied agentrig binary and 68 ms without it, against 130 and
65 ms with sha2 0.10 before.

## Blockers

None.

## Next action

Commit through the hook, then `just propose-pin HEAD` and `just feature-merge`.
