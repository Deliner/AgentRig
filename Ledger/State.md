# State

## Focus

The user requires hard enforcement of named-if-condition, function-lines and parameter-count, and repair of all existing violations. D021 records blocking defaults at the existing limits: named conditions, at most 40 nonblank function lines and 4 counted inputs. P001 remains paused for the authorized maintenance priority.

## Workspace

Branch: feature/strict-language-lint

Revision: 823b3dc

This is the pre-change baseline, not a claim that the working tree is unchanged. Reconcile the recorded branch and baseline with current Git before acting. The coherent VAC combines strict policy with the refactoring necessary to pass it.

## Progress

Rust runtime, Python tests and benchmark have been refactored into cohesive operations with named branch reasons. The linter's scope and counting semantics are preserved. Repository configuration and new consumer templates use error severity and hard numeric limits. I017 exercises rejecting each violation and accepting exact boundaries in both languages. Current documentation identifies D021 as superseding D017's advisory rollout.

## Verification

163 native behavioral tests passed after enabling strict defaults. Clippy, mypy and memory/oracle validation passed. The three language rules report zero findings; remaining advisory diagnostics concern the existing skill file and directory sizes. The refactored benchmark ran successfully on both installed examples. The final staged and integration gates are executed by the normal commit/merge workflow; inspect their actual results before declaring delivery.

## Blockers

None observed. Existing consumer configurations are not silently rewritten; newly initialized consumers receive the new defaults.

## Next action

Inspect and commit this coherent VAC through the full staged gate, correcting failures. Once committed on its clean feature branch, run just feature-merge and verify clean master plus the retained feature reference. If Git already confirms that integration, the strict-language task is delivered; follow new authorized instructions rather than restarting it or P001.
