---
name: norm-or-choice
description: Classify durable project knowledge as either an invariant with an executable oracle or a decision with explicit context, alternatives, and rationale. Use before adding or changing any long-lived entry in Ledger/Decisions.md, Ledger/Invariants.md, AGENTS.md, feature workflow, or code policy.
---

# Classify a norm or a choice

Apply this procedure before writing durable knowledge at the code, process, research, or agent level.

1. Identify the claim first. A norm claims a predicate must hold in every reachable state in its stated scope. A choice selects one of several admissible alternatives in a stated context.
2. Negate the proposed statement. If it claims a must-hold predicate and the negation produces an observable repository state that a test can expose, record an invariant in `Ledger/Invariants.md`, add its detail under `Ledger/Invariants/`, and link its exact marked pytest oracle.
3. If the negation is another admissible alternative whose selection depends on context and rationale, record a decision in `Ledger/Decisions.md` and add its detail under `Ledger/Decisions/`. Its detail must state `Context`, `Chosen`, `Rejected`, `Rationale`, and `Consequences`.
4. If both results occur, create two linked records. The decision explains why; the invariant names the state that must remain true and links its oracle.

An invariant without a passing executable oracle is prohibited in this repository. Do not hide a wish or an external requirement that lacks a repository oracle as an indexed invariant; keep its source visible in the governing instruction. A decision is guarded against context drift by its detail file, grounded `Applies in` links, and delivery through `AGENTS.md` or the feature-execution skill whenever an agent must act on it.

A decision may create an invariant, and that invariant may cite the decision. Do not infer rationale from an invariant. Record that rationale as a separate decision.

Revise explicitly. Replace an invariant only through a recorded decision, updating its predicate, detail, and exact test together. Supersede a decision only with a new decision that names the changed context and the displaced decision. Keep the old indexed decision and its detail; the repository checker rejects modification or deletion of committed decisions and removal of prior application links. Identify successors in the Decisions index so resume does not apply obsolete policy. Invariant records may change under an explicit decision; they are not append-only decision history.

Ordinary Plan updates use the existing delivery policy: changing feature status, recording a discovered prerequisite, or adding authorized follow-ups does not itself require a new policy decision. Record a durable decision only when the update makes a genuine contextual choice that needs its own rationale. Feature details still state outcomes and acceptance, with Delivery context for the source of new work, blockers, and verification.

At any scale, use the same test: an invariant can be violated; a decision can be reconsidered. They have different failure modes and must not share one record.

D019 permits only its five deleted Python application links to migrate to the named Rust owners. Decision IDs, statements and detail files remain immutable; this exception does not authorize other link removal.
