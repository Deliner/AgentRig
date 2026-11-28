---
name: size-the-read-buffer
description: Give a vector the length that a read should fill, or read to the end of the input, instead of reading into an empty vector whose capacity the read ignores.
---

# Size the buffer that a read fills

`read` and `read_exact` fill as many bytes as the slice they get is long, not
as many as its vector can hold. `Vec::with_capacity(len)` creates a vector of
length zero, so a read into it reads nothing and returns 0, which the caller
takes for the end of the input.

1. Read how many bytes the code means to read: an exact count, at most a
   count, or the whole input.
2. For an exact count, create the vector with that length and fill all of it:
   `let mut data = vec![0; len];` and `reader.read_exact(&mut data)?`.
3. For at most a count, read through `take`:
   `reader.take(limit).read_to_end(&mut data)?`. For the whole input, call
   `read_to_end` or `read_to_string`, which grow the vector themselves.
4. Rerun the reported lint check and the affected behavior checks.

No exception keeps the read: a read into an empty vector reads nothing, so do
not suppress the lint.

## Rule binding

The rig's Clippy profile denies `clippy::read_zero_byte_vec` in the library,
binaries and examples of the configured Cargo manifest; tests are outside the
profile. Clippy reports a read into a vector created empty, such as by
`Vec::new()` or `Vec::with_capacity(..)`, before anything gives it a length,
and the lint check reports each message as an error naming this skill. Do not
weaken the profile merely to pass. Apply complexity-discipline; stop when every
read gets a buffer of the length it should fill.
