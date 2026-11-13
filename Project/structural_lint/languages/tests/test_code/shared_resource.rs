use super::measured;
use crate::structural_lint::rule_identities::reference::Reference;

#[test]
fn rust_flags_tests_that_bind_a_fixed_port() {
    let source = "#[cfg(test)]
mod tests {
    #[test]
    fn serves() {
        let listener = TcpListener::bind(\"127.0.0.1:8080\").unwrap();
        let socket = std::net::UdpSocket::bind((\"0.0.0.0\", 9000)).unwrap();
    }
}
";
    assert_eq!(
        measured(source, Reference::FixedTestPort),
        [(5, "bind".into()), (6, "bind".into())]
    );
}

#[test]
fn rust_skips_port_zero_named_hostless_computed_connected_justified_and_product_sockets() {
    let source = "#[cfg(test)]
mod tests {
    #[test]
    fn serves() {
        let listener = TcpListener::bind(\"127.0.0.1:0\").unwrap();
        let socket = UdpSocket::bind((\"0.0.0.0\", 0)).unwrap();
        let named = TcpListener::bind(ADDRESS).unwrap();
        let host = TcpListener::bind(\"localhost\").unwrap();
        let computed = TcpListener::bind((\"127.0.0.1\", port)).unwrap();
        let client = TcpStream::connect(\"127.0.0.1:8080\").unwrap();
        // DECISION: D001
        TcpListener::bind(\"127.0.0.1:8080\").unwrap();
    }
}
fn product() {
    TcpListener::bind(\"0.0.0.0:8080\").unwrap();
}
";
    assert!(measured(source, Reference::FixedTestPort).is_empty());
}
