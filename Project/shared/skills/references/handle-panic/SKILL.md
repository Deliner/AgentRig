---
name: handle-panic
description: Return or handle a Rust error instead of unwrap, expect, unwrap_err, expect_err, resume_unwind, a panic macro, a string slice by byte range or a process exit outside main, or record why the panic is unreachable or the exit intended.
---

# Handle the error instead of panicking

`.unwrap()` and `.expect(...)` abort the program when an Option is None or a
Result is Err, `.unwrap_err()` and `.expect_err(...)` abort when a Result is
Ok, `resume_unwind` restarts a caught panic, and `panic!`, `todo!`,
`unimplemented!` and `unreachable!` abort outright. In shipped, non-test code
each one turns a condition the caller could recover from into a crash it
cannot.

1. Read what the call can fail on. For a Result or Option, propagate the
   failure with `?`, return a `Result`, or match the variants and handle both.
2. For an absent value that is a genuine invariant, restructure so the types
   carry the guarantee, or record the reasoning as a decision with
   edit-decisions and put its exception `// DECISION: DNNN RULE`, RULE being
   the rule ID the finding names, alone on the line directly above the call.
   List it in the decision as `[exception RULE](FILE)`; the user approves a new
   exception through propose-protected-change.
3. For `todo!`/`unimplemented!`, finish the work or return an error; for
   `unreachable!`, encode the impossible case in the types so the branch does
   not exist; for `resume_unwind`, return the failure as an error unless the
   panic must cross the boundary unchanged, and record why as in step 2.
4. Rerun the reported lint check and the affected behavior checks.

An exception naming this rule, `panic-path` by default, on the line directly
above the call keeps it, attesting the panic is unreachable; a bare `DECISION:`
or `INVARIANT:` reference lifts nothing. When the project's Clippy profile
forbids the panic, an outer `#[allow]` or `#[expect]` of that lint on an
enclosing item, such as `clippy::expect_used` for `.expect(...)`, covers every
such panic inside the item without an exception above each call. The attribute
itself is an `allow-justification` finding: give it its own exception
`// DECISION: DNNN allow-justification` directly above it, as justify-allow
describes.

## Clippy profile

The rig's Clippy profile denies `clippy::unwrap_used`, `clippy::expect_used`,
`clippy::panic`, `clippy::todo`, `clippy::unimplemented`,
`clippy::unreachable`, `clippy::string_slice` and `clippy::exit` in the
library, binaries and examples of the configured Cargo manifest; tests are
outside the profile. The lint check reports each Clippy message as an error
naming this skill. The first six report the calls and macros that the steps
above repair; the other two report:

- `clippy::string_slice`: a string sliced by a byte range, such as
  `&text[start..]`, which panics when a bound falls inside a character of
  several bytes. Take the part with `text.get(start..)` and handle `None`, or
  cut with `split_once`, `strip_prefix` or `split_at_checked`.
- `clippy::exit`: a `std::process::exit` call outside `main`, which ends the
  process without running the destructors on any thread's stack, so no caller
  can handle the failure. Return the failure to `main` as an error or an
  `ExitCode`, and let `main` end the process.

When the panic or the exit is the intended outcome, such as slice bounds that
`char_indices` of the same string places at character boundaries, the
covering `#[expect]` of the reported lint described above, on the smallest
item that holds the call and with its own `allow-justification` exception,
keeps it. Do not weaken the profile merely to pass.

## Rule binding

The measurement flags every Rust `.unwrap()`, `.expect(...)`, `.unwrap_err()`
and `.expect_err(...)` call, every `resume_unwind` call through a path or a bare
name, and every `panic!`, `todo!`, `unimplemented!` and `unreachable!` macro,
in source, macro arguments and `macro_rules!` bodies, outside test code, unless
an outer `#[allow]` or `#[expect]` of its Clippy lint (`unwrap_used`,
`expect_used`, `panic`, `todo`, `unimplemented`, `unreachable`) on an enclosing
item covers it. An exception naming this rule on the line directly above the
reported line lifts the finding. Test code is out of scope. Rerun the reported
lint check and relevant behavior checks. Do not weaken the rule or its
selectors merely to pass. Apply complexity-discipline; stop when the error is
returned or handled, or an approved exception or a covering attribute with its
own approved exception lifts the finding.
