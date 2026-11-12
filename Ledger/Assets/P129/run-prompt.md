You work on a copy of a Rust repository. Your working directory {RUN} holds `Project/`, the copy, and `./structure`,
which prints `agentrig lint-structure Project` for it. Apply the skill below to `Project/` until its stop rule holds.

- Build with `cargo check --manifest-path Project/Cargo.toml --all-targets --offline`.
- Keep the map in `variants/result.map`. `python3 -I {TOOLS}/check.py {RUN}` applies it to the original file list,
  compares the result with `Project/`, checks the naming constraints and the description texts, and ends with
  `check ok` when everything holds.
- `Project/shared/syntax/tree-sitter-rust` is vendored code, and the symbolic links under `Project/shared/skills` are
  aliases. The map and the check ignore both; leave them in place inside their parent directories.
- Work only inside {RUN}; {TOOLS} is read-only. The copy has no Git and needs no network.
- Finish with the build passing and `check ok`, then reply with the first line of `./structure` and the number of
  edits you kept.
