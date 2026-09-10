# State

## Focus

The user requested committing and publishing all accumulated authorized changes
to GitHub. Resume feature/backlog-card-review, run the normal commit gate,
integrate through feature-merge and push the base and retained feature branch.
This is maintenance authorization; it does not select Backlog implementation.

B017 was explicitly approved and saved in [Backlog/017.md](Backlog/017.md),
including the agreed Tmp listing and three-way hash verification amendment.
K8 requires the current Plan card to be Active for every VAC commit. The user
resolved archival timing: the ready card remains Active until its branch merges
successfully, then MCP completes and archives it automatically.
The latest amendment specifies automatic migration verified in a temporary
clone before the main project, current-date fallback for unknown historical
dates, and encrypted MCP hash references committed in Git inside the project.
The user resolved the key source: embed it in the supplied MCP binary, with no
separate user key setup. This is not a guarantee against deliberate bypass.
P017 is complete in [Archive/Plan/017.md](Archive/Plan/017.md).
Plan is empty; no product implementation is selected.

## Workspace

Branch: feature/backlog-card-review

Revision: a129830

Installed runtime stays pinned to 9aee4ae. Work remains uncommitted, including
earlier skill changes, Backlog revisions and the archived audit. Local and fetched
origin/master are a8b0ff8; the retained feature branch is a129830. No Git operation
is in progress. No new commit, merge, push or runtime upgrade has been performed.

## Progress

All 16 Backlog cards were reviewed one at a time, explicitly approved in chat
and saved with their criterion IDs preserved. P017 retains the audit procedure,
completed checklist, outcome and material questions before implementation.
Its index and detail moved to Archive; current Plan no longer selects work.

Latest agreements:
- B014 defines registered project operations, permitted basic commands and
  temporary scenarios through the existing execution hook. Native reading,
  searching, listing and editing require no Just routing. Tool structure and
  registry checks run before merge, with no added VAC procedures.
- B015 makes review configurable. Disabled review has no model calls or report
  requirement; enabled review failures block merge. Mechanical checks still run.
- B016 binds criteria to module-owned test registries and actual results. K9
  covers stable card/criterion IDs, matching headers/files/indexes and references
  including archived cards. Formal validity does not prove semantic correctness.
- B016 K10 selects affected-file tests on VAC, affected feature/module trees
  across the branch on merge, and the full suite on explicit request. This is
  future behavior; the installed runtime still uses its existing full merge gate.
- B017 defines Ledger MCP operations, Plan transitions and dates, Archive,
  Docs with an index, Tmp without an index but with filename listing, and a
  Markdown placement rule. Every Ledger Markdown file carries its content hash;
  verification compares actual content, the embedded value and the MCP reference.
  References now live encrypted in a project-owned MCP directory and are committed
  in Git with matching files; separate directory access protection is not required.
  The encryption key is embedded in the supplied MCP binary; the binary package
  supports writing and checking references without separate user key setup.
  An embedded key is extractable, so deliberate tamper resistance is not promised.
  The hash field itself is excluded from hashing. These are future
  requirements, not installed capabilities. Backlog index and inventory include B017.
  K8 adds a mandatory pre-commit Active-card check. Ready cards remain Active
  through the final VAC and until successful merge of their branch; failed or
  cancelled merge does not complete or archive them. Completion date follows
  successful merge. Manual archival also requires successful merge.
  K9 requires successful automatic migration and verification in a temporary clone
  before migration in the main project. Existing dates are retained; unknown
  historical dates use the current migration date. No migration has been run.

Earlier authorized changes to 12 canonical skills remain: complexity-discipline,
the five memory skills, name-if-condition, reduce-parameters, refactor-long-function,
refactor-large-file, refactor-large-directory and repair. Their intent-focused
rewrite is implemented. At the user's explicit request, edit-backlog's drafting
core now requires concrete operations/reactions, their triggers, inputs, effects,
outputs and relevant constraints without inventing additional behavior.
The latest correction requires checking operation coverage across elements,
collections, relationships and states. A prohibited former workflow must retain
its necessary capabilities through the replacement. This is a reasoning check,
not a requirement to invent every possible operation or create a separate report.
It distinguishes a reusable reasoning scheme from a domain-specific feature card.
The agreement, saving and execution sections are unchanged by these amendments.
The user subsequently identified semantic anchoring in the operation examples;
that limitation was acknowledged. No further generalization of the skill was
performed while drafting or saving B017. Preserve this concern for follow-up.
Canonical bodies are exposed through .agents/skills; preserve these edits.
Local drafting experiment reports remain at
.tmp/card-structures-20260909/abc-three-cards/. The voxel-rust audit was read-only.

## Verification

Before publication on 2026-09-11, the current working tree passed memory-check
and git diff --check. The configured hooksPath is .githooks. The commit gate,
integration gate and remote publication remain to be performed.

The initial B017 save matched the approved card with the Tmp listing and hash
amendment after storage formatting and navigation links, preserving K1-K7.
That save passed existing memory-check and diff whitespace checks.
The K8 and subsequent merge/archive amendments passed memory-check and diff
whitespace checks. The merge amendment updated completion, archival and date
semantics together. The migration/date/encrypted-storage amendment also passed
memory-check and diff whitespace checks. The embedded-key choice is recorded
as a requirement; no binary or cryptographic implementation has been changed.

B001–B016 saves and P017 progress updates passed existing memory-check and
whitespace checks at their respective boundaries. The last B016 body matched
the approved card plus its K9 amendment exactly after storage formatting and
navigation links. All 16 audit checklist items are complete.
Memory-check passed after updating the archive index, moved links and final State.

The latest edit-backlog correction passed quick_validate and diff whitespace
checks. The .agents entry resolves to the canonical source. Agreement, saving
and execution sections matched their contents before this correction.
No independent model evaluation was run for this change.

Earlier verification of the 12 skills, before this amendment: quick_validate passed,
approved B core compared exactly, and 20 installation-manifest/memory-route tests
passed in 10.58s using the candidate fixture. These checks were not rerun here.
They establish format, packaging and routing, not model decision quality.

No implementation acceptance for the Backlog features is claimed. B001's reported
consumer failure was inspected in source only, not reproduced. Context telemetry,
new command-routing behavior and future lint/review requirements were not
implemented or tested by this audit.

## Blockers

None for publication. The user confirmed the reference document's deletion;
its inventory entry and remaining links have been removed.

None for the completed audit. Material product questions remain in the Backlog
cards and archived P017, including unsupported analysis, context telemetry,
command classification, cleanup events, test discovery and evidence reuse.
Resolve behavior-changing questions before selecting those features for delivery.

## Next action

Stage the accumulated authorized Ledger and canonical skill changes, commit through the configured
hook, run feature-merge and push master plus the retained feature branch.
Verify the remote revisions and clean working tree before reporting publication.
The question about the missing Tmp edit operation remains unanswered; preserve
the existing card text until that unrelated change is clarified.
Plan remains empty. Preserve unrelated uncommitted changes.
