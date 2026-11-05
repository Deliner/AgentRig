---
name: fail-closed-enforcement
description: Make every enforcement verdict fail closed, so an error or an unknown on the decision path denies with its reason instead of silently allowing.
---

# Deny unless the check positively passed

An enforcement site (a hook, guard, gate or merge check) decides whether an
action may proceed. Its failure modes are asymmetric: a wrong denial costs a
retry, a wrong permission silently defeats the policy. Shape the code so the
permissive answer can only come from a successful positive check.

1. The verdict is a two-variant type, `Allow` and `Deny` (or the project's
   names), where `Deny` carries its reason. `Allow` is produced only by the
   branch that observed the positive condition; no default, fallback or
   `unwrap_or` yields it.
2. An error or an unknown on the decision path becomes `Deny` with the cause
   attached: a failing Git call, an unresolved reference, an unreadable file,
   a missing field. Do not collapse it into `Ok(false)`, `Ok(true)`,
   `unwrap_or_default()` or an `Err(_) => {}` arm. The cause travels to the
   final decision and into the output the caller sees, so a denial is
   diagnosable and is never mistaken for the policy itself.
3. Each verdict site has a negative test per contract refusal: the test makes
   the check fail (an unresolvable tip, a broken repository, an absent file)
   and asserts through the public surface that the action is denied. A site
   tested only on its happy path is unverified.
4. A `Result<bool>` verdict is permitted when the error propagates with `?`
   and the boolean is the positive check's answer. Mapping an error arm to a
   boolean is the fail-open shape the `silent-error` rule reports in functions
   returning `Result<bool>`.

Stop when every path to `Allow` passes through one positive check and every
other path denies with a reason. Apply complexity-discipline: one verdict type
per enforcement owner, not a parallel policy layer.

## Enforcement

This is a review norm with an advisory hook route: editing an enforcement
owner names this skill and never blocks. The delivered route covers the
configured source tree; narrow it in `.agentrig/hooks.yaml` to the directories
that own verdicts (AgentRig routes `Project/agent_hooks/**` and
`Project/change_delivery/**`). The polarity of a fallback is not decidable from
syntax: `unwrap_or(false)` is a safe default in one enforcement zone and a
fail-open hole in another, so no rule flags it; only the visible `Result<bool>`
collapse ships as a rule. The repository's own guards are the precedent: the
reference guard fails its transaction on a Git error (I021) and the commit and
hook guards deny with the reason.
