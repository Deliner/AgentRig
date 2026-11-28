# State

## Focus

P203 (B138 AC1–AC3): the finding of each lint of the rig's Clippy profile names a skill that names the lint.
VAC 3 of 4: the 15 existing skills that take 24 profile lints name each one as `clippy::LINT`, with the rewrite and
when an exception is allowed.

## Workspace

Branch: feature/clippy-lint-skills
Revision: 56c4d81

- Each of the 15 skills gains a "Clippy profile" section before its rule binding; type-driven-discipline extends its
  Enforcement section. Lints: handle-panic (panic, todo, unimplemented, unreachable, expect_used, unwrap_used,
  string_slice, exit), remove-debug (dbg_macro), expect-the-panic (should_panic_without_expect), name-the-states
  (option_option, struct_excessive_bools), name-the-placeholders (uninlined_format_args), match-every-variant
  (wildcard_enum_match_arm), justify-allow (allow_attributes), name-the-target (default_trait_access),
  handle-discarded (let_underscore_must_use), subtract-checked (unchecked_time_subtraction), handle-error
  (map_err_ignore), justify-unsafe (multiple_unsafe_ops_per_block), release-the-guard
  (significant_drop_in_scrutinee), destructure-the-fields (rest_pat_in_fully_bound_structs,
  missing_fields_in_debug), type-driven-discipline (partial_pub_fields).
- Where a repair scope grows, the skill description and its architecture purposes say so.

## Progress

- VAC 1 (29508a5): name-if-condition moved into the naming group.
- VAC 2 (56c4d81): 12 new skills for the 13 profile lints that no skill covered, shipped by init.
- VAC 3 written and checked; commit pending. Group and top-level purposes name the widened hazards.
- VAC 4: profile.rs maps each lint to its skill and a Clippy finding carries that skill in lint output and --json;
  tests for AC1–AC3 and for the completeness of the mapping; D240 narrows the configuration-skill clause of D239.

## Verification

VAC 3: `just lint` passes;
`just run test -- init_delivers_the_universal_capabilities_of_this_repository` (I020) passes; an rg of
`clippy::NAME` over the skills finds all 37 profile lints.

## Blockers

None.

## Next action

Commit VAC 3, then map each profile lint to its skill in code and carry it in the finding (VAC 4).
