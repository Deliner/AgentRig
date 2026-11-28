use std::path::{Path, PathBuf};

struct Lint {
    name: &'static str,
    skill: &'static str,
}

// DECISION: D060
// DECISION: D068
// DECISION: D107
// DECISION: D109
// DECISION: D127
// DECISION: D129
// DECISION: D132
// DECISION: D137
// DECISION: D152
// DECISION: D153
// DECISION: D180
// DECISION: D239
// DECISION: D240
const PROFILE: &[Lint] = &[
    Lint {
        name: "panic",
        skill: "handle-panic",
    },
    Lint {
        name: "todo",
        skill: "handle-panic",
    },
    Lint {
        name: "unimplemented",
        skill: "handle-panic",
    },
    Lint {
        name: "dbg_macro",
        skill: "remove-debug",
    },
    Lint {
        name: "unreachable",
        skill: "handle-panic",
    },
    Lint {
        name: "expect_used",
        skill: "handle-panic",
    },
    Lint {
        name: "unwrap_used",
        skill: "handle-panic",
    },
    Lint {
        name: "let_underscore_must_use",
        skill: "handle-discarded",
    },
    Lint {
        name: "exit",
        skill: "handle-panic",
    },
    Lint {
        name: "should_panic_without_expect",
        skill: "expect-the-panic",
    },
    Lint {
        name: "missing_fields_in_debug",
        skill: "destructure-the-fields",
    },
    Lint {
        name: "rest_pat_in_fully_bound_structs",
        skill: "destructure-the-fields",
    },
    Lint {
        name: "self_named_module_files",
        skill: "move-into-mod-rs",
    },
    Lint {
        name: "print_stdout",
        skill: "print-through-one-owner",
    },
    Lint {
        name: "print_stderr",
        skill: "print-through-one-owner",
    },
    Lint {
        name: "string_slice",
        skill: "handle-panic",
    },
    Lint {
        name: "iter_over_hash_type",
        skill: "iterate-in-a-fixed-order",
    },
    Lint {
        name: "collection_is_never_read",
        skill: "read-or-remove-the-collection",
    },
    Lint {
        name: "read_zero_byte_vec",
        skill: "size-the-read-buffer",
    },
    Lint {
        name: "unchecked_time_subtraction",
        skill: "subtract-checked",
    },
    Lint {
        name: "map_err_ignore",
        skill: "handle-error",
    },
    Lint {
        name: "partial_pub_fields",
        skill: "type-driven-discipline",
    },
    Lint {
        name: "struct_excessive_bools",
        skill: "name-the-states",
    },
    Lint {
        name: "option_option",
        skill: "name-the-states",
    },
    Lint {
        name: "useless_let_if_seq",
        skill: "bind-with-if",
    },
    Lint {
        name: "return_self_not_must_use",
        skill: "mark-must-use",
    },
    Lint {
        name: "multiple_unsafe_ops_per_block",
        skill: "justify-unsafe",
    },
    Lint {
        name: "significant_drop_in_scrutinee",
        skill: "release-the-guard",
    },
    Lint {
        name: "uninlined_format_args",
        skill: "name-the-placeholders",
    },
    Lint {
        name: "wildcard_enum_match_arm",
        skill: "match-every-variant",
    },
    Lint {
        name: "as_conversions",
        skill: "convert-without-as",
    },
    Lint {
        name: "allow_attributes",
        skill: "justify-allow",
    },
    Lint {
        name: "default_trait_access",
        skill: "name-the-target",
    },
    Lint {
        name: "suspicious_xor_used_as_pow",
        skill: "write-the-power",
    },
    Lint {
        name: "host_endian_bytes",
        skill: "name-the-byte-order",
    },
    Lint {
        name: "large_stack_arrays",
        skill: "box-the-large-array",
    },
    Lint {
        name: "maybe_infinite_iter",
        skill: "bound-the-iterator",
    },
];

pub fn denials() -> Vec<String> {
    let profile = PROFILE.iter().map(|lint| format!("clippy::{}", lint.name));
    let denied = std::iter::once("warnings".to_owned()).chain(profile);
    denied.flat_map(|lint| ["-D".to_owned(), lint]).collect()
}

pub fn skill(rule: &str, skills: &Path) -> Option<PathBuf> {
    let name = rule.strip_prefix("clippy::")?;
    let lint = PROFILE.iter().find(|lint| lint.name == name)?;
    Some(skills.join(lint.skill).join("SKILL.md"))
}
