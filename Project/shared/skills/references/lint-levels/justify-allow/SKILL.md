---
name: justify-allow
description: Remove a lint suppression (allow, expect, a cfg_attr form, an allow level in a TOML lint table or an allowing flag in the rig's flag list) and fix the finding, or record why it is suppressed with a reference.
---

# Remove a suppression, or record why the lint is silenced

A broad `#[allow(...)]` or `#![allow(...)]` switches a compiler or Clippy lint
off for its scope, and `#[expect(...)]` silences it the same way, adding only an
error once the lint stops firing. A conditional `#[cfg_attr(cond, allow(...))]`,
`#[cfg_attr(cond, expect(...))]` or `#[cfg_attr(cond, ignore)]` is the same
suppression behind a configuration predicate, and a lint set to `"allow"` or
`{ level = "allow" }` under `[lints.*]` or `[workspace.lints.*]` in Cargo.toml
silences it for a whole package. An item starting with `-A`, `--allow` or
`--cap-lints` in the `rustflags` list of the rig's `.agentrig/rust.yaml`
silences lints in every build the rig runs, and `--cap-lints` turns the gate's
denied lints back into warnings. In every form the gate stays green while the
finding it would raise goes unaddressed: an agent can quiet a rule instead of
fixing the code. Every suppression must be removed with its finding fixed, or
carry a DECISION: or INVARIANT: reference on the line directly above the
attribute, the manifest key or the flag item, recording why the suppression is
right.

1. Read the suppression and the code it covers; establish which lint it
   silences and what that lint would report. A suppression claims the finding
   need not be fixed.
2. Prefer removing the suppression and fixing the finding: delete the dead
   code, repair the unused binding, handle the flagged case, or narrow the
   construct.
3. Narrow a still-needed suppression to the smallest scope and the exact lint it
   must cover, rather than a crate-wide `#![allow]`, a package-wide manifest
   level, a flag in the rig's list or a blanket `cfg_attr`.
4. When the suppression is genuinely required, such as a deprecated upstream
   API with no replacement or a generated construct, record the reason as a
   decision or bind it to an existing invariant, then put its `DECISION: DNNN`
   or `INVARIANT: INNN` reference comment on the line directly above the
   attribute (`//`), the manifest key or the flag item (`#`). Write a Rust
   suppression as
   `#[expect(...)]`, not `#[allow(...)]`: the shared Clippy profile denies
   `allow_attributes`, and an expectation fails once the lint stops firing.
5. Rerun the reported lint check and the affected behavior checks.

The reference is the only accepted form: an exact `DECISION: DNNN` or
`INVARIANT: INNN` comment on the line directly above the attribute or the key.
By project convention the rule does not read a `reason = ".."` argument, even
when it states a sound reason: record that reason as the decision or invariant
and reference it. Any other comment does not justify the suppression either. Do
not silence the finding with a plain comment, a `reason` string, or by widening
the rule.

## Rule binding

The measurement flags every Rust `#[allow(...)]`, `#![allow(...)]`,
`#[expect(...)]` or `#![expect(...)]` attribute, every `#[cfg_attr(...)]` whose
direct argument after a comma is `allow`, `expect` or `ignore` (a conditional
`ignore` reports under disabled-test), and every lint set to `"allow"` or
`{ level = "allow" }` under a `[lints.*]` or `[workspace.lints.*]` table of a
TOML manifest, and every item of a YAML `rustflags` sequence starting with
`-A`, `--allow` or `--cap-lints`, without a DECISION: or INVARIANT: reference
comment on the line directly above the attribute, the key or the item. The
flag list is read only when the rule selects `.agentrig/rust.yaml` and the
`.yaml` extension. No other comment and no `reason`
argument is read. Rerun the reported lint check and relevant behavior checks. Do
not weaken the rule or its selectors merely to pass. Apply
complexity-discipline; stop when the suppression is removed or justified.
