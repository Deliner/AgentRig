use std::collections::BTreeMap;

pub fn skills() -> BTreeMap<&'static str, String> {
    let mut result = BTreeMap::new();
    result.insert(
        "complexity-discipline",
        include_str!("../../../assets/complexity-discipline.md").to_owned(),
    );
    for (name, purpose, instructions) in [
        (
            "split-large-file",
            "Resolve a file-size diagnostic while preserving behavior.",
            "Read the measured file and its callers. Separate coherent responsibilities into existing modules or small new modules with explicit interfaces. Preserve initialization order and public APIs. Do not move arbitrary line ranges, compress formatting, or raise limits to hide the finding. Verify affected behavior.",
        ),
        (
            "organize-directory",
            "Resolve a directory-entry diagnostic through meaningful grouping.",
            "Group files by an existing responsibility or feature. Update imports, resource paths and build discovery after moving files. Avoid numbered buckets or one-file directories that only satisfy the counter. Verify that runtime and test discovery still find the moved content.",
        ),
        (
            "name-if-condition",
            "Resolve a named-if-condition diagnostic without changing evaluation semantics.",
            "Name the decision with one meaningful variable before the branch. Preserve short-circuiting, evaluation count, side effects and the scope of bindings. Keep loop and comprehension decisions at their original evaluation point. Do not precompute an elif condition before earlier branches or conceal the expression behind a meaningless name.",
        ),
        (
            "refactor-long-function",
            "Resolve a function-lines diagnostic with cohesive extractions.",
            "Extract a responsibility with explicit inputs and result, using existing helpers where they fit. Preserve early returns, exceptions, resource lifetimes, async behavior and mutation order. Check callers and affected behavior. Avoid arbitrary line-count splits or formatting compression.",
        ),
        (
            "reduce-parameters",
            "Resolve a parameter-count diagnostic without hiding dependencies.",
            "Remove redundant inputs or group values that already form a meaningful domain concept. Update callers, defaults and keyword interfaces consistently. Preserve receiver and constructor semantics. Do not replace named arguments with an untyped bag, varargs or implicit global state merely to reduce the count.",
        ),
        (
            "repair",
            "Repair scaffold configuration and failed project checks.",
            "Read the reported field, check and original tool output. Correct the cause and rerun that check. Preserve the intended behavior and user changes. Do not weaken thresholds or exclude files merely to hide a failure. Use config-check for settings and doctor for missing dependencies.",
        ),
        (
            "edit-plan",
            "Edit the product delivery plan and its feature details.",
            "Keep stable IDs and user-facing outcomes. Acceptance describes observable results. Preserve prerequisites; active and complete items require completed prerequisites. Record blockers and resumption conditions for paused features, and actual verification for completion. Do not replace a required outcome with a smaller one.",
        ),
        (
            "edit-decisions",
            "Record a contextual choice in project memory.",
            "Use Context, Chosen, Rejected, Rationale and Consequences. Preserve committed decision identity and detail history. Link actual applications and mark supported source files with the decision ID. Supersede prior choices explicitly through a new decision.",
        ),
        (
            "edit-invariants",
            "Link an observable invariant to an executable project test.",
            "Use Predicate and Oracle sections. Link the real test function and its source marker. Configure the matching test target and gate check in worker.toml. A passing discovery check proves the target exists; only running the test checks the predicate.",
        ),
        (
            "edit-state",
            "Keep a compact recovery snapshot of current work.",
            "Use Focus, Workspace, Progress, Verification, Blockers and Next action. Record observed facts and pending work. Branch: `name` and Revision: `hash` lines allow resume to compare Git facts. State is a snapshot; inspect actual Git and current instructions before acting. Do not create a second task database.",
        ),
    ] {
        result.insert(
            name,
            format!("---\nname: {name}\ndescription: {purpose}\n---\n\n{instructions}\n"),
        );
    }
    result
}
pub fn memory() -> BTreeMap<&'static str, String> {
    let mut files = BTreeMap::new();
    for (name, columns) in [
        (
            "Plan",
            "ID | Status | Depends on | Feature | User capability",
        ),
        ("Decisions", "ID | Decision | Applies in"),
        ("Invariants", "ID | Invariant | Enforced by"),
    ] {
        let separator = columns
            .split('|')
            .map(|_| "---")
            .collect::<Vec<_>>()
            .join(" | ");
        files.insert(
            name,
            format!("# {name}\n\n| {columns} |\n| {separator} |\n"),
        );
    }
    files.insert(
        "State",
        "# State\n\n".to_owned()
            + &[
                "Focus",
                "Workspace",
                "Progress",
                "Verification",
                "Blockers",
                "Next action",
            ]
            .iter()
            .map(|heading| format!("## {heading}\n\nNot recorded.\n"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    files
}
