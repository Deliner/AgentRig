use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_tests_that_bind_a_fixed_port_or_write_a_fixed_path() {
    let source = "#[cfg(test)]
mod tests {
    #[test]
    fn serves() {
        let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();
        let socket = std::net::UdpSocket::bind((\"0.0.0.0\", 9000)).unwrap();
        File::create(\"output.txt\").unwrap();
        std::fs::write(\"data.json\", \"{}\").unwrap();
        fs::create_dir_all(r\"target/cache\").unwrap();
        fs::create_dir(\"fixture\").unwrap();
    }
}
";
    assert_eq!(
        measured(source, Reference::FixedTestPort),
        [(5, "bind".into()), (6, "bind".into())]
    );
    assert_eq!(
        measured(source, Reference::FixedTestPath),
        [
            (7, "create".into()),
            (8, "write".into()),
            (9, "create_dir_all".into()),
            (10, "create_dir".into()),
        ]
    );
}

#[test]
fn rust_skips_port_zero_named_hostless_computed_read_justified_and_product_resources() {
    let source = "#[cfg(test)]
mod tests {
    #[test]
    fn serves() {
        let listener = TcpListener::bind(\"127.0.0.1:0\").unwrap();
        let socket = UdpSocket::bind((\"0.0.0.0\", 0)).unwrap();
        let named = TcpListener::bind(ADDRESS).unwrap();
        let host = TcpListener::bind(\"localhost\").unwrap();
        File::create(dir.join(\"output.txt\")).unwrap();
        fs::write(path, \"{}\").unwrap();
        File::open(\"input.txt\").unwrap();
        // DECISION: D001
        TcpListener::bind(\"127.0.0.1:8080\").unwrap();
    }
}
fn product() {
    TcpListener::bind(\"0.0.0.0:8080\").unwrap();
    fs::write(\"state.json\", \"{}\").unwrap();
}
";
    assert!(measured(source, Reference::FixedTestPort).is_empty());
    assert!(measured(source, Reference::FixedTestPath).is_empty());
}
