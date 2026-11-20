# State

## Focus

P167 (B174) VAC 4: the rig's lint exceptions name their rule and are listed in their decisions (D219, D220).

## Workspace

Branch: feature/named-exceptions
Revision: 1d92698418a40829a2347711c01690e28285cfd7

## Progress

- VAC 1 to 3 and the pin of 5f87bcb are committed.
- 34 markers that lift a finding became `DECISION: DNNN RULE` exceptions, each listed as `[exception RULE](FILE)`.
- The old `kept comparison` and `kept job directory` links of D156 and D092 became exception links.
- The comparison in `command_runner/sandbox.rs` now cites D156, which allows it, instead of D193.
- D221 records the deprecated `mach_timebase_info` binding that D066 cited without covering it.
- Accidental lifts of `same-type-parameters` are removed in code: `sandbox` takes the scratch `TempDir`,
  `same_text` takes the selected target and `lowered` takes the baseline by value.

## Verification

- With marker lifts disabled for the check only, lint reports the 34 named sites and no `same-type-parameters`.
- `just candidate check --only memory` passes on the exception links.

## Blockers

The commit guard refuses the 34 new exceptions until the user approves them with their own `just approve`.

## Next action

Commit VAC 4 after the approval, then VAC 5: lint lifts only the named rule and reports an exception without a finding.
