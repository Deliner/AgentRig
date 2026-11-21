# State

## Focus

P193 (B200): the architecture rule parses each source once and resolves dependencies on several threads.

## Workspace

Branch: feature/faster-rules
Revision: 686a7af84a7363ca73c7633c4e44267c81121ffc

## Progress

VAC 1 saved B200 and selected it as P193; VAC 2 resolves the dependencies of each file on several threads. VAC 3 adds
`Parsed` in structural_lint/architecture/contract_check/analysis.rs: one parallel parse of every maintained Rust
source and every scope file, split into the Rust sources the resolvers read and the other scope files; the scope view
joins both without a second parse. Parse issues are cloned before the resolution gaps of each file, as before, so
`Issue` derives Clone. The temporary timer is removed.

## Verification

Release candidate on AgentRig, interleaved warm runs of the architecture rule: master analysis 298, 305, 309 and
314 ms; single parse with parallel resolution 173 to 198 ms. Candidate lint and Clippy are clean.

## Blockers

None.

## Next action

Commit VAC 3, then archive P193 and B200 with the timings and run the full test suite and lint before feature-merge.
