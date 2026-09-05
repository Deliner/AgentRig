---
name: edit-plan
description: Maintain outcome-based feature delivery, prerequisites and acceptance.
---

Plan rows use stable PNNN IDs, Status, Depends on, Feature and User capability. Details contain Feature, User capability and Acceptance; paused/complete entries also require Delivery with blocker/resumption context or verification evidence. Use row order for priority, actual prerequisites and at most one active feature; zero active is valid. Add future outcomes only from current requirements, observed blockers or authorized instructions. Preserve required acceptance; do not defer it and claim completion. Use execute-plan-feature for VAC and branch handoffs.
