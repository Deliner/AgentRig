# State

## Focus

P205 (B209) VAC A: `.agentrig/project.yaml` accepts an optional queue `key` of 2 to 10 capital Latin letters, the
first half of AC1, so that a pin can install a runtime that reads the rig module once it names `key: RIG`.

## Workspace

Branch: feature/card-queue
Revision: ab6fef22036e0ff95d2032757f314286bd251ac1

## Progress

- `Project/project_config/settings_schema/key.rs`: the `Key` type checks the form in its only constructor
  (`TryFrom<String>`); its hand-written `Deserialize` goes through that check, as `Flag` does, because the rig
  dependency analyzer refuses serde `try_from` and `into` type paths.
- `Config` gains `key: Option<Key>`; init still writes no key, so the I023 comparison with the rig is unchanged.
- The settings_schema contract and the project_setup README name the key.

## Verification

- `project_config::tests::settings::a_queue_key_is_two_to_ten_capital_latin_letters`: config-check accepts AB, RIG
  and ABCDEFGHIJ and refuses R, ABCDEFGHIJK, rig, R1G, RIG-1 and РИГ, naming `.agentrig/project.yaml: key:`.
- `just run rustfmt` and `just lint` pass on this content; the commit gate runs the affected groups.

## Blockers

None for this VAC.

## Next action

Commit this VAC, then pause P205: the pin of this VAC and the user's approval of the rig module with `key: RIG`
(proposal in the session scratchpad as b209-proposal) come before init can require and write the key.
