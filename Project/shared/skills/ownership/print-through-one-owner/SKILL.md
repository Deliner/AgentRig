---
name: print-through-one-owner
description: Remove a leftover print or eprint macro, or write output through the one module that owns the terminal, or record why a module owns the terminal.
---

# Print through the owner of the terminal

`print!`, `println!`, `eprint!` and `eprintln!` write straight to the
standard output and error of the process. Spread over a program, they mix
leftover diagnostics into its real output, break output that another program
parses, such as JSON, and leave no single place to redirect, silence or test
what the program writes.

1. Read whether the print is a leftover diagnostic or output the program must
   produce.
2. Remove a leftover diagnostic. When the information is still needed, return
   it to the caller, add it to an error or assert it in a test.
3. Send output the program must produce through the module that owns the
   terminal: call its functions instead of the macros, or let the code that
   formats the output write to a `&mut impl Write` that its caller passes.
   Writing to `io::stdout()` directly is the same output without an owner; do
   not do it to pass.
4. When the project has no such owner yet, make one module the owner: record
   why as a decision with edit-decisions and put
   `#![expect(clippy::print_stdout, clippy::print_stderr)]` at the top of that
   module, with its exception `// DECISION: DNNN allow-justification` alone on
   the line directly above the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes. Other code
   prints only through that module.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::print_stdout` and
`clippy::print_stderr` in the library, binaries and examples of the configured
Cargo manifest; tests are outside the profile. Clippy reports `print!` and
`println!` under `print_stdout` and `eprint!` and `eprintln!` under
`print_stderr`, and the lint check reports each message as an error naming this
skill. Do not weaken the profile merely to pass. Apply complexity-discipline;
stop when the output goes through its owner or the leftover is removed, or an
approved exception lifts the finding.
