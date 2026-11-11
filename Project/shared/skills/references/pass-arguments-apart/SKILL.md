---
name: pass-arguments-apart
description: Pass each value to a launched program apart from the shell command string instead of substituting it into that string, or record why the string is meant to be shell code.
---

# Keep data out of the shell command string

A shell started with `-c`, `/c` or `-Command` parses its command string as
code. A path, name or message substituted into that string is parsed too: a
space splits it, a quote closes the surrounding string, and `;`, `&`, `|` or
`$( )` start another command. The command works on ordinary values and breaks,
or runs something else, on the first value that carries such a character.

1. Read what the string does with the value. If the shell only starts one
   program, drop the shell: `Command::new(program).arg(value)` passes the value
   as one argument that nothing parses.
2. If the script needs the shell, such as for a pipe, a redirect or a builtin,
   keep the script a literal and pass each value apart from it:
   - `sh`, `bash`, `zsh`, `dash` and `ksh` take positional parameters after
     the script. The first one becomes `$0`, so pass a name before the values
     and quote each parameter in the script:
     `.args(["-c", r#"grep -- "$1" "$2""#, "sh", pattern, path])`.
   - `fish` puts the values after the script into `$argv`.
   - `powershell` and `pwsh` join the arguments after `-Command` into the
     command text, so they are not apart. Set each value with
     `.env("NAME", value)` and read `$env:NAME` in the literal script.
   - `cmd` has no parameters for `/c` and expands `%NAME%` before it parses
     the line, so an environment variable is not apart either. Run the
     program directly or do the work in Rust.
3. Keep the substitution only when the string is meant to be shell code, such
   as a task, hook or alias command that the user or configuration writes, or
   when every substituted value is a number or is quoted for that shell by a
   quoting function. Record the reasoning as an owned `DECISION:`/`INVARIANT:`
   reference on the line directly above the `Command::new` call.
4. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags a Rust `Command::new` call whose first argument is a
string literal naming `sh`, `bash`, `zsh`, `dash`, `ksh`, `fish`, `cmd`,
`powershell` or `pwsh` by its last path component, with or without `.exe`,
when the `.arg` and `.args([..])` calls chained on it put `-c`, `/c`,
`-Command` or a lowercase short-flag cluster ending in `c`, such as `-lc`,
directly before an argument that is neither a string literal nor an uppercase
constant name. A command built in steps through a variable, a chain inside a
macro argument, test code and a call with a `DECISION:` or `INVARIANT:`
reference on the line directly above are out of scope. Do not weaken the rule
or its selectors merely to pass. Apply complexity-discipline; stop when every
value reaches the program apart from the command string or the shell code is
justified by an owned reference.
