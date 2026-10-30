---
name: remove-secret
description: Remove a hardcoded credential from source and move it to configuration or a secret store.
---

# Take the credential out of the source

A string literal matched a known credential format: a private-key block, or an
AWS, GitHub, Google or Slack token. A credential committed to source is exposed
to everyone who can read the history; the fix is to remove it, not to hide the
finding.

1. Confirm what matched. A real credential must be removed and rotated; a value
   that only resembles the format (a fixture, an example, documentation) still
   does not belong in a scanned source literal.
2. For a real credential, delete the literal and read the value at run time from
   configuration, the environment, or a secret store that the deployment owns.
   The source keeps the lookup, never the value.
3. Rotate the exposed credential. Anything committed is compromised and must be
   reissued, even after the literal is gone from the working tree.
4. For a value that only resembles the format and must stay in a test, build it
   so no single source literal is a complete pattern, or move it to a resource
   file the scanner does not read. Do not weaken the rule to admit it.
5. Rerun the reported lint check and the affected behavior checks.

## Rule binding

The measurement flags every string literal matching a fixed set of unambiguous
credential formats. Rerun the reported lint check and relevant behavior checks.
Do not weaken the rule or its selectors, or relocate a real secret merely to pass.
Apply complexity-discipline; stop when the finding is resolved and the credential
is out of the source and rotated.
