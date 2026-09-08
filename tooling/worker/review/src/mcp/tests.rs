#[path = "../../testing/mod.rs"]
pub mod support;
use review_runner::mcp::Server;
use serde_json::json;
use support::Fixture;

#[test]
fn mcp_lists_configured_tools_and_validates_arguments() {
    let fixture = Fixture::new("pass");
    let mut server = Server::new(&fixture.0.path().join("config.yaml")).unwrap();
    let init = server.message(json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25"}})).unwrap();
    assert_eq!(
        init["result"]["capabilities"]["tools"]["listChanged"],
        false
    );
    let list = server
        .message(json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}))
        .unwrap();
    assert_eq!(list["result"]["tools"][0]["name"], "review_code");
    assert_eq!(
        list["result"]["tools"][0]["inputSchema"]["additionalProperties"],
        false
    );
    let rejected = server.message(json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"review_code","arguments":{"root":".","base":"HEAD","candidate":"HEAD","tool":"override"}}})).unwrap();
    assert_eq!(rejected["error"]["code"], -32602);
    assert!(
        server
            .message(json!({"jsonrpc":"2.0","method":"notifications/initialized"}))
            .is_none()
    );
}
