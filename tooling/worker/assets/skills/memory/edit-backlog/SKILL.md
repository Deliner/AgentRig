---
name: edit-backlog
description: >
  Draft and agree feature cards in chat, then save the approved version in
  Backlog. Use when capturing an idea, clarifying requirements, revising an
  existing card or preparing features for autonomous Plan execution.
---

# Agree the outcome before saving or executing

Use proactively for concrete ideas outside current work and when the user asks
to prepare or revise a feature. Read the configured Backlog and relevant Plan
entries to avoid duplicates and preserve context. Distinguish historical
observations, current evidence and hypotheses. Do not investigate merely to fill
a card or revive rejected ideas without new evidence or instruction.

## Draft in chat

Keep new cards and proposed requirement changes in chat until the user explicitly
approves saving the shown version. Discussing or preparing a card, praise for an
idea, silence and elapsed time are not approval. An explicit instruction to save
an already shown version is sufficient; no special phrase or second confirmation
is required. Do not save unapproved drafts in State or another file instead.

Show the complete current card in the user's language. After revisions, also show
the substantive difference: previous behavior -> new behavior -> consequence.
Do not reinterpret the approved content while saving. Already authorized
mechanical link, ID and delivery-evidence updates do not reopen agreement on
unchanged requirements.

A future idea may stay short. Elaborate only cards selected for discussion or
upcoming execution. Offer unrelated ideas without stopping authorized work to
seek approval; keep an unapproved draft in chat and continue the current task.

## Preserve meaning and limit scope

Describe business behavior: who acts, when, what they do and what they receive.
The user does not choose code patterns, layers, modules or other implementation
details. Ask about observable consequences only when an unresolved choice changes
the required result. Reuse answers already supplied.

Every required behavior, constraint and acceptance criterion needs a basis in
the user's request, an agreed clarification or an applicable project contract.
Keep the source or short faithful wording in Context; attribute any additional
basis next to the affected rule. Preserve the user's terms through delivery and
acceptance. Explain terms only where ambiguity affects behavior. Label proposed
examples and agent interpretations as unconfirmed until agreed.

Do not invent mandatory inputs, validation, limits, normalization, formats,
fallbacks, options or side effects. Each addition needs a concrete basis above
or a necessary technical condition for the agreed behavior to work. Convention,
possible usefulness and extra confidence are insufficient. Technical necessity
does not authorize a new user-visible restriction. Discard unsupported additions
instead of asking the user to approve every imagined precaution. Preserve
existing applicable constraints.

Expose consequential ambiguity through concrete different outcomes; do not guess
the user's answer. The agent owns internal implementation choices and selects
the simplest sufficient solution.

## Card format and acceptance

Keep the existing Context, Proposal and Expected benefit sections so the shown
body can be saved without another translation or memory schema. Use this content
for an elaborated feature, omitting inapplicable optional fields:

```markdown
# Title

## Context

**Original request and basis:** The user's problem, request and relevant sources.
**Terms:** Clarify only terms with consequential ambiguity.

## Proposal

**User journey:** Actor and starting situation -> action -> observable result.

| ID | Situation / action | Expected observable result |
| --- | --- | --- |
| K1 | A concrete reproducible case | What distinguishes success from failure |

**Agreed constraints:** Only established boundaries and requirements.
**Open questions:** Unresolved behavior, or None.
**Dependencies:** Established feature prerequisites, if any.

## Expected benefit

What capability or improvement the user receives and why it matters.
```

The scenario table is the acceptance contract, with stable local criterion IDs.
Do not maintain a separately paraphrased behavior specification. Include a full
representative journey and, where actual ambiguity exists, a contrasting case
that separates the intended behavior from a similar wrong result. Use concrete
sample inputs, outputs or documents where helpful; an example does not authorize
inventing general rules.

Criteria must be reproducible and observable without the original chat. Avoid
undefined claims such as "correct", "convenient" or "reliable". Explain guarantees
through their trigger, required circumstances and visible result. An agent
instruction and a runtime operation provide different guarantees; do not silently
substitute them. Failure cases must follow established requirements or credible
observed failures, not an invented exhaustive catalogue. Technical verification
methods belong to the implementing or checking agent.

## Save the agreed version

Read paths.memory in agentrig.yaml. Maintain Backlog.md with
ID | Idea | Expected benefit rows and Backlog/NNN.md cards. Allocate a stable
unused BNNN from index and cards; never renumber other ideas. Preserve an existing
card's identity and agreed context. Save the approved body with only necessary
ID/navigation metadata, update index and applicable architecture inventories
together, and use the existing memory check. Ordinary editing does not require
just write. Do not copy secrets or full transcripts. Report failed saves.

Backlog has no execution priority. Saving is not implementation authorization.
Existing saved ideas are not automatically agreed for delivery under this
process; refine selected cards in chat before changing their contract.
This is a skill-guided policy, not a new approval registry or proof of
machine-verified understanding.

## Hand off and check the same contract

All new product outcomes pass through Backlog before Plan. Promote only cards
with agreed behavior and acceptance, resolved behavior-changing questions, and
execution and order selected by the user. Use edit-plan and execute-plan-feature.
One instruction to execute a selected set authorizes the whole set; do not ask
again before each card. Plan still covers current or resumed work, and Backlog
is not an automatic queue.

Plan references the agreed Backlog contract rather than maintaining a second
independently rewritten specification. Preserve wording and criterion IDs; use
existing Git history to identify the agreed version for delivery and review.
Saving a card or changing its status does not establish user agreement.
Do not change acceptance to fit the implementation.

For acceptance by the executor, a subagent or a reviewer, provide the agreed
card version, implementation/artifacts and means to exercise the scenarios.
Use delegation or review only under existing authorization and workflow.
Report each criterion as met, violated or not verified, with concrete evidence.
Not verified is not success. Passing implementation-authored tests alone does
not establish all criteria. Also inspect new rejection conditions, transformations,
formats and side effects against agreed scope. The verifier follows the same
contract and applicable project rules without inventing feature requirements.
Do not claim that a review covered criteria outside its scope.

Keep required work in the current task. For explicit deferral, preserve its
requirements, progress, blocker, branch and resume condition before removing it
from Plan, and reconcile dependencies. Preserve source context and delivery links;
replace live Plan links with Archive links on completion. Interruption alone is
not deferral. Follow edit-plan for automatic completed-card archival.
