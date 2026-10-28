---
name: develop-tools
description: Develop, repair or remove project tools and catalog commands, including findings of the tools check.
---

# Keep one registered owner for each capability

A tool is a file with a shebang, an executable bit or a `tools.recognize` match.
Permanent capabilities run through the command catalog by name; temporary work
runs through `scratch` in its branch task area and needs no registration.

1. Find existing capabilities with `just list`: active commands with purpose,
   parameters, location, owner and documentation, then removed tools.
2. Establish which required result is missing. Use an existing capability when
   it suffices, extend the matching one when it almost does, and create a new
   one only when none can provide the result.
3. Place a permanent tool under `tools.locations` in the directory that owns
   its responsibility. Register it as a catalog command with `purpose` and,
   where details exist, `doc`; list it in `tools.entries` only when Git, Just
   or another tool starts it directly.
4. Update the instructions, checks and documentation that name it.

Do not add unrequested modes, generality or a duplicate that differs only in
name or location. Repeated temporary work becomes a permanent capability.

## Removal

Remove a permanent tool only after establishing that no required capability
depends on it; rare use alone is not a reason. Update or remove its consumers,
delete the file and its catalog command, and record it under `tools.removed`
with `purpose`, `reason`, `path` and the last `revision` containing it. Omit
`revision` for a never committed file; the listing then marks it unrecoverable.

## Rule binding

The `tools` check runs before merge, not on VAC commits. It reports tools
outside their locations or unregistered, commands without `purpose`, missing
argument or documentation paths, entries matching no tool, and removal records
that are incomplete, collide with active names, still exist, lack the revision
or are still referenced outside memory. Repair the cause, then rerun the
reported command; reading this skill does not replace the repair.
