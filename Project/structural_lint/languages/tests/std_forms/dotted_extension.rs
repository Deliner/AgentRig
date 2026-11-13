use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_an_extension_literal_written_with_its_dot() {
    let source = "fn renamed(path: &Path, mut buffer: PathBuf) -> PathBuf {
    buffer.set_extension(\".json\");
    path.with_extension(r\".tar.gz\")
}
";
    assert_eq!(
        measured(source, Reference::DottedExtension),
        [(2, "set_extension".into()), (3, "with_extension".into())]
    );
}

#[test]
fn rust_skips_plain_empty_variable_other_method_justified_and_test_extensions() {
    let source = "fn renamed(path: &Path, extension: &str) -> PathBuf {
    let plain = path.with_extension(\"json\");
    let empty = path.with_extension(\"\");
    let named = path.with_extension(extension);
    let other = path.with_file_name(\".hidden\");
    // DECISION: D001
    let justified = path.with_extension(\".json\");
    plain
}
#[cfg(test)]
mod tests {
    fn check(path: &Path) -> PathBuf {
        path.with_extension(\".json\")
    }
}
";
    assert!(measured(source, Reference::DottedExtension).is_empty());
}
