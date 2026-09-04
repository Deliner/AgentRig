---
name: refactor-long-function
description: Reduce functions, methods or closures reported by function-lines while preserving their behavior.
---

Read the reported function and its callers; fix syntax first if parsing failed. The measurement spans the signature through the body and counts nonblank lines including comments, nested definitions and docstrings, but excludes decorators.

Find a cohesive responsibility that can be given a useful name and extracted without changing order, state, exceptions or return behavior. Keep related logic together; avoid arbitrary slicing, line compression or deleting explanations to meet a number.

Use the lowest sufficient change under complexity-discipline. Rerun just lint and checks exercising the changed behavior; do not raise thresholds merely to pass.
