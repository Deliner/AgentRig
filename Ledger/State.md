# State

## Focus

P181 (B188): the maintained file list is built once per memory check and without the directory walk where only
files are needed. VAC 2 splits a files-only listing out of the inventory and uses it in the memory check, the gate
file list and the architecture check.

## Workspace

Branch: feature/file-listing
Revision: b7d42609f2fd1fcec774bc998fef1bb3409265bf

## Progress

maintained_files now returns only files with one git ls-files; maintained_inventory keeps files and directories
for lint and its reports. The memory check lists once and passes the list to references and exceptions;
invariant_ids keeps references within 40 lines. Temporary timers are reverted.

## Verification

Full suite: 614 passed, as on master e7ad50e. `just lint`: no findings. Memory check median in a new project
112 and 120 ms before, 68 and 67 ms after. Git processes: memory check 6 to 1, gate list in AgentRig 9 to 1.

## Blockers

None.

## Next action

Commit VAC 2, then record Delivery, archive P181 and B188 and run `just feature-merge`.
