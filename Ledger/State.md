# State

## Focus

P152 (B153 AC1–AC4), VAC2: pin the runtime that refuses shell forms just would not run as written, and archive P152
and B153.

## Workspace

Branch: feature/hook-shell-parse
Revision: 07db0a44eed259ba449fe966e5f7d1949b051cf1

## Progress

- VAC1 `07db0a4` passed the gate; `just bootstrap HEAD` installed it and wrote it to stable.txt.
- P152 and B153 moved to Archive with Delivery and the criterion table; D201 links point to the archived cards.

## Verification

- Live zsh after the pin: `=(…)` and `*(e:…:)` were refused and created no file, the same text in quotes passed the
  hook, `just jobs bootstrap` and `./tooling/x/just lint` were refused, and an agent `--root` reached `cat`.
- `just check --only memory` passes.

## Blockers

None.

## Next action

Commit VAC2, run `just feature-merge`, then start P153 (B154).
