# State

## Focus

On 2026-10-27 the user deleted unneeded Backlog cards and selected the rest
for execution: [Plan](Plan.md) orders B001 → B003 → B002 → B008 → B009 →
B014. B001, B003, B002, B008 and B009 are complete as
[P018](Archive/Plan/018.md), [P019](Archive/Plan/019.md),
[P021](Archive/Plan/021.md), [P025](Archive/Plan/025.md) and
[P026](Archive/Plan/026.md). P027 (tool registry, [B014](Backlog/014.md)) is
the remaining pending card. References to the removed review runner are
assessed by the executor at acceptance ([D033](Decisions/033.md)).

## Workspace

Branch: feature/target-state

Revision: c67408a1ca266475f80c79c448a3221594acd0f1

The machine is macOS (Darwin 25.5.0) with Homebrew flock. stable.txt pins
c1dde2c (D047 hook coordination), installed by `just bootstrap` under
.cache/development. Change the pin and run bootstrap in one shell command:
a pin without its install makes every hook fail. A stray branch
feature/init-probe at a14f284 remains from an install probe; guard-reference
refuses its deletion. Agent shells lack cargo and uv on PATH;
prefix `PATH=/Users/admin/.cargo/bin:...` inside `just write --`.

## Progress

P026 ran the K5 set fixed in c67408a: a clean haiku agent without repository
access applied name-if-condition to seven real conditions. Six passed K1–K4;
select_base's combined condition was named by listing its parts (K2 partial).
Step 4 of the skill now asks for the joint fact; a blind re-run of cases 1,
4 and 5 passed. Per-case results are in Archive/Plan/026.md. The installed
runtime (c1dde2c) predates P025's memory-check reference scan, which the
candidate already has.

## Verification

A scratch lint consumer with named-if-condition reported all eight original
inline conditions and accepted every corrected and re-run result. The
archive VAC changes the skill text and Ledger records only.

## Blockers

None. memory-check cannot run inside `just read`.

## Next action

Start P027: establish B014's start questions (tool and base-invocation
recognition rules, the task-completion event, the order of saving results
and cleaning logs) from the current command catalog, hook routes and job
logs; record them in P027 Delivery, or a blocker if a choice needs the user.
