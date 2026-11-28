---
name: name-the-byte-order
description: Replace to_ne_bytes and from_ne_bytes with the little-endian or big-endian form that the format of the data requires, or record why the bytes must follow the host's order.
---

# Name the byte order of the bytes

`to_ne_bytes` and `from_ne_bytes` use the byte order of the machine that runs
the code. Bytes that one machine writes to a file, a socket or a hash input
read back as another number on a machine with the other order, and the code
does not show which order its format expects.

1. Find the format the bytes belong to: a file format, a network protocol, a
   hash or checksum input, or memory shared with other code on the same
   machine.
2. Call the form the format specifies: `to_le_bytes` and `from_le_bytes` for
   little-endian, `to_be_bytes` and `from_be_bytes` for big-endian, the
   network order of most protocols. For a format the project owns, choose one
   order and use it in every reader and writer; on the common little-endian
   machines, x86-64 and ARM64, the `le` form keeps the bytes the `ne` form
   wrote.
3. When the bytes must follow the host's order, such as a buffer passed to a C
   library or to the operating system on the same machine, record the
   reasoning as a decision with edit-decisions and put
   `#[expect(clippy::host_endian_bytes)]` on the smallest item that holds the
   call, with its exception `// DECISION: DNNN allow-justification` alone on
   the line directly above the attribute. List it in the decision as
   `[exception allow-justification](FILE)`; the user approves a new exception
   through propose-protected-change, as justify-allow describes.
4. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The rig's Clippy profile denies `clippy::host_endian_bytes` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports every call of `to_ne_bytes` and `from_ne_bytes`, and
the lint check reports each message as an error naming this skill. Do not
weaken the profile merely to pass. Apply complexity-discipline; stop when every
conversion names its byte order, or an approved exception lifts the finding.
