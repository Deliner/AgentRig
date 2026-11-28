# State

## Focus

P205 (B209), step 1: the runtime accepts an optional queue key. project.yaml may hold key, 2 to 10 uppercase Latin
letters, and init --key writes it after the same check, refusing another value before writing anything. The key stays
optional until the user approves key: RIG in project.yaml and --key RIG in the installer.

## Workspace

Branch: feature/RIG-209-one-card-per-task
Revision: 3ee14ce

## Progress

- Key in Project/project_config/settings_schema/key.rs is checked on deserialization and in init; Config.key is
  optional; init usage, CLI help and the project_setup README name --key.
- Key deserializes through its check by hand, like Flag, since the architecture analyzer refuses
  serde(try_from = ...) as incomplete dependency analysis.
- Tests: project_key_takes_two_to_ten_uppercase_latin_letters and
  init_writes_the_queue_key_it_is_given_and_refuses_another_before_writing.

## Verification

- just run test -- key: 6 passed, including both new tests.
- just run rustfmt and just run lint passed.

## Blockers

None.

## Next action

Commit, then run just propose-pin HEAD under the consent of the user and commit the pin with State.
