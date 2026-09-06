---
name: repair
description: Repair reported configuration or verification failures.
---

Read the reported rule/check, source location and original tool output. Use config-check for settings, lint-rules for supported targets/languages, and doctor for installation problems. Apply the named skill, correct the cause and execute the reported RERUN command; where shell operations require Just, use `just run write -- <RERUN command>`. For a gate check, `just check --only CHECK_ID` repeats that stage; preserve `--staged` when reported. Preserve behavior, selector intent and user work; do not raise thresholds or exclude files merely to pass. New rules need implemented measurements, declared capabilities, behavioral tests and repair guidance. Numeric warning must be below error. Directory-entries rejects extensions; directory-architecture requires explicit source extensions and architecture settings. Unsupported selected languages are configuration errors.


If the same failure recurs after applying the skill, compare the actual output with the attempted repair. `just report` shows consecutive failures of a check and the guidance presented; it does not establish that a skill was read or that failures share a cause. When the instruction or diagnostic caused the mistake, correct its canonical source and verify the previously failing scenario. Keep that evidence in the current VAC context and commit, without another registry or mandatory checkpoint. Do not change guidance just because an unrelated failure has the same check ID.
