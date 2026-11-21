# State

## Focus

P174, B181: make the gate faster and the rig code simpler without changing behavior. VAC 5 reads the files that
the lint exception finder compares through one `Repository::blobs` process instead of one Git process per file side.

## Workspace

Branch: feature/faster-gate
Revision: 421f5d2a9446c72d7f20f020fa9cb5aeb8030e56

## Progress

VAC 2 to 4 batched oracle discovery and committed blob reads and shared the SHA-256 digest. A timing probe showed
`init` spends 1.3 of 1.65 s in recording approval, whose exception finder started `git cat-file` for each changed
file on both sides. VAC 5 plans every read first, streams the blobs and parses each as it arrives, so findings,
their order and the refusal of unreadable objects stay the same. The commit guard uses the same finder.

## Verification

After VAC 4 the full suite of 614 tests took 93.5 s against 103 s in B181. With VAC 5 a warm `init` in the probe took
571 ms instead of 1653 ms; `just test project_setup` passed 58 in 22.8 s, was 37 s, and `just test
change_delivery::git` 32 in 33.3 s, was 37.5 s. `just lint`, `just run rustfmt` and `just run clippy` are clean.

## Blockers

None.

## Next action

Commit VAC 5, then profile feature-merge at 1.6 s, the hooked commit at 0.6 s and feature-start at 0.3 s.
