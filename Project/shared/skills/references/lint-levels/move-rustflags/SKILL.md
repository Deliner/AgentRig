---
name: move-rustflags
description: Move the compiler flags of a rustflags key in a Cargo configuration into the rig's flag list and delete the key.
---

# Keep the project's compiler flags in the rig's list

The rig runs every cargo command with exactly the `rustflags` list of its `rust`
module, `.agentrig/rust.yaml`, and ignores `RUSTFLAGS` and every Cargo
configuration. A `rustflags` key in a `.cargo/config.toml`, under `build`,
`target.<triple>` or `target.'cfg(..)'`, still acts in plain cargo and the
editor, so the gate and those builds silently compile different code.

1. Read the key and its flags. A string value holds flags separated by
   whitespace; an array holds one flag per item.
2. Append each flag of `build.rustflags`, in order, as its own item of the
   `rustflags` list in `.agentrig/rust.yaml`, then delete the key, and the
   `build` table when it is left empty.
3. A flag under `target.<triple>` or `target.'cfg(..)'` applies to that target
   only, while the rig's list applies to every build. Move it like a build flag
   when every target the project builds needs it. Otherwise stop and report the
   conflict: the rig's list has no per-target form.
4. A flag that allows lints or caps their level, such as `-A`, `--allow` or
   `--cap-lints`, needs a `DECISION:` or `INVARIANT:` reference on the line
   directly above its item in the list; apply justify-allow to it.
5. Rerun the reported lint check and the affected build and test checks.

## Rule binding

The measurement flags every `rustflags` key in any table of a TOML file that
the rule selects, by default `**/.cargo/config.toml`. No comment justifies the
key: the rig applies only its own list. Do not weaken the rule or its selectors
merely to pass. Apply complexity-discipline; stop when the flags live in the
rig's list and no selected file has the key.
