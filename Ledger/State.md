# State

## Focus

P203 (B138 AC1–AC3): the finding of each lint of the rig's Clippy profile names a skill that names the lint.
VAC 2 of 4: write a skill for each profile lint that no existing skill covers, and ship the new skills with init.

## Workspace

Branch: feature/clippy-lint-skills
Revision: 29508a5

- 12 new skills for 13 lints; partial_pub_fields goes to type-driven-discipline, whose item 5 already analyses it
  (AC2: no second skill with the same analysis).
  - New group Project/shared/skills/values: convert-without-as (as_conversions), write-the-power
    (suspicious_xor_used_as_pow), name-the-byte-order (host_endian_bytes), size-the-read-buffer
    (read_zero_byte_vec), box-the-large-array (large_stack_arrays), iterate-in-a-fixed-order (iter_over_hash_type),
    read-or-remove-the-collection (collection_is_never_read), bound-the-iterator (maybe_infinite_iter),
    bind-with-if (useless_let_if_seq).
  - signature/mark-must-use (return_self_not_must_use), ownership/print-through-one-owner (print_stdout,
    print_stderr), refactor/move-into-mod-rs (self_named_module_files).
- Each new skill has its architecture.yaml, a flat alias symlink in Project/shared/skills and an entry in
  Project/project_setup/bundle/skills.rs; the group contracts and the top contract describe them.

## Progress

- VAC 1 committed as 29508a5: name-if-condition moved into the naming group, leaving room for the values group.
- VAC 2 written and checked; commit pending.
- Lints for existing skills: handle-panic (panic, todo, unimplemented, unreachable, expect_used, unwrap_used,
  string_slice, exit), remove-debug (dbg_macro), expect-the-panic (should_panic_without_expect), name-the-states
  (option_option, struct_excessive_bools), name-the-placeholders (uninlined_format_args), match-every-variant
  (wildcard_enum_match_arm), justify-allow (allow_attributes), name-the-target (default_trait_access),
  handle-discarded (let_underscore_must_use), subtract-checked (unchecked_time_subtraction), handle-error
  (map_err_ignore), justify-unsafe (multiple_unsafe_ops_per_block), release-the-guard
  (significant_drop_in_scrutinee), destructure-the-fields (rest_pat_in_fully_bound_structs,
  missing_fields_in_debug), type-driven-discipline (partial_pub_fields).
- VAC 3: those 15 skills name their lints as `clippy::LINT` with the rewrite and when an exception is allowed.
- VAC 4: profile.rs maps each lint to its skill and a Clippy finding carries that skill in lint output and --json;
  tests for AC1–AC3 and for the completeness of the mapping; D240 narrows the configuration-skill clause of D239.

## Verification

VAC 2: `just config-check` reports the configuration valid, `just lint` passes and
`just run test -- init_delivers_the_universal_capabilities_of_this_repository` passes (I020: init ships the same
skill set as Project/shared/skills, which now holds the 12 new skills).

## Blockers

None.

## Next action

Commit VAC 2, then extend the 15 existing skills (VAC 3).
