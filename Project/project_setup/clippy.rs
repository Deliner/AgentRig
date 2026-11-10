use super::cargo::Layout;

// DECISION: D132
// DECISION: D137
// DECISION: D152
// DECISION: D153
const PROFILE: &[&str] = &[
    "panic",
    "todo",
    "unimplemented",
    "dbg_macro",
    "unreachable",
    "expect_used",
    "unwrap_used",
    "let_underscore_must_use",
    "exit",
    "should_panic_without_expect",
    "missing_fields_in_debug",
    "rest_pat_in_fully_bound_structs",
    "self_named_module_files",
    "print_stdout",
    "print_stderr",
    "string_slice",
    "iter_over_hash_type",
    "collection_is_never_read",
    "read_zero_byte_vec",
    "unchecked_time_subtraction",
    "map_err_ignore",
    "partial_pub_fields",
    "struct_excessive_bools",
    "option_option",
    "useless_let_if_seq",
    "return_self_not_must_use",
    "multiple_unsafe_ops_per_block",
    "significant_drop_in_scrutinee",
    "uninlined_format_args",
    "wildcard_enum_match_arm",
    "as_conversions",
    "allow_attributes",
];

pub(super) fn argv(rust: &Layout) -> Vec<String> {
    let mut argv: Vec<String> = ["cargo", "clippy", "--manifest-path"]
        .into_iter()
        .map(String::from)
        .collect();
    argv.push(rust.manifest.clone());
    argv.extend(["--", "-D", "warnings"].map(String::from));
    for lint in PROFILE {
        argv.push("-D".into());
        argv.push(format!("clippy::{lint}"));
    }
    argv
}
