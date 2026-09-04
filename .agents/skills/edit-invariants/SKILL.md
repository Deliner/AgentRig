---
name: edit-invariants
description: Edit Ledger/Invariants.md or invariant details with an observable predicate and exact executable oracle.
---

# Edit invariants

- Apply norm-or-choice. State behavior that must hold and whose violation a repository test can expose; keep contextual rationale in Decisions.
- Use a stable INNN ID, an indexed matching detail with Predicate and Oracle sections, and a link to the exact pytest function.
- Put # INVARIANT: INNN immediately before that test definition. Test the observable failure and valid behavior, not merely the wording of the rule.
- Replace a predicate only under an explicit recorded decision. Change the index, detail, implementation, and oracle consistently; these records are not append-only decision history.
- Run the relevant test before claiming the invariant holds. Wishes or external claims without an executable repository oracle stay outside the invariant index.
