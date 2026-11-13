use crate::structural_lint::languages::analyze;
use crate::structural_lint::rule_identities::kind::Kind;
use crate::structural_lint::rule_identities::ownership::Ownership;
use crate::structural_lint::rule_identities::reference::Reference;
use std::path::Path;

const GUARD: Kind = Kind::Ownership(Ownership::GuardAcrossAwait);
const BLOCKING: Kind = Kind::Reference(Reference::BlockingInAsync);

// DECISION: D117
// DECISION: D118
fn findings(kind: Kind, source: &str) -> Vec<(usize, String)> {
    let analysis = analyze(Path::new("source.rs"), source, &[]).unwrap();
    assert_eq!(analysis.parse_error, None);
    analysis
        .measurements
        .into_iter()
        .filter(|item| item.kind == kind)
        .map(|item| (item.line, item.symbol))
        .collect()
}

#[test]
fn rust_flags_std_lock_guards_alive_at_a_later_await() {
    let source = "async fn plain(state: &Mutex<u8>) {
    let guard = state.lock().unwrap();
    send(*guard).await;
}
async fn variants(lock: &RwLock<u8>) -> Result<()> {
    let reader = lock.read()?;
    let mut writer = lock.write().expect(\"held\");
    let other = lock.read().unwrap_or_else(|error| error.into_inner());
    allow_err!(conn.send(1).await);
    Ok(())
}
";
    let expected = [(2, "guard"), (6, "reader"), (7, "writer"), (8, "other")]
        .map(|(line, symbol)| (line, symbol.to_string()));
    assert_eq!(findings(GUARD, source), expected);
}

#[test]
fn rust_accepts_guards_released_before_await_and_non_guard_bindings() {
    let source = "async fn released(state: &Mutex<u8>) {
    let guard = state.lock().unwrap();
    drop(guard);
    send(1).await;
    {
        let inner = state.lock().unwrap();
        touch(&inner);
    }
    send(2).await;
    let _ = state.lock().unwrap();
    send(3).await;
}
async fn exempt(state: &tokio::sync::Mutex<u8>, std: &Mutex<u8>, file: &mut File) {
    let guard = state.lock().await;
    let value = std.lock().unwrap().clone();
    let count = file.read(&mut buffer)?;
    // DECISION: D001
    let held = std.lock().unwrap();
    send(*guard + value + count + *held).await;
}
async fn deferred(std: &Mutex<u8>) {
    let spawned = std.lock().unwrap();
    let task = async move { send(4).await };
    let later = || async { send(5).await };
}
fn sync(state: &Mutex<u8>) {
    let guard = state.lock().unwrap();
    touch(&guard);
}
#[cfg(test)]
mod tests {
    async fn check(state: &Mutex<u8>) {
        let guard = state.lock().unwrap();
        send(*guard).await;
    }
}
";
    assert_eq!(findings(GUARD, source), []);
}

#[test]
fn rust_flags_thread_sleep_and_block_on_inside_async_code() {
    let source = "use std::thread::{self, sleep};
async fn poll_status(client: &Client) -> Status {
    loop {
        if let Some(status) = client.status().await {
            return status;
        }
        std::thread::sleep(Duration::from_millis(200));
    }
}
async fn load(runtime: &tokio::runtime::Runtime) -> Config {
    runtime.block_on(fetch_config())
}
impl Worker {
    async fn tick(&self) {
        thread::sleep(PAUSE);
        sleep(PAUSE);
        let task = async {
            futures::executor::block_on(job());
            block_on::<u8>(job());
        };
    }
}
";
    let expected = [
        (7, "sleep"),
        (11, "block_on"),
        (15, "sleep"),
        (16, "sleep"),
        (18, "block_on"),
        (19, "block_on"),
    ]
    .map(|(line, symbol)| (line, symbol.to_string()));
    assert_eq!(findings(BLOCKING, source), expected);
}

#[test]
fn rust_accepts_blocking_calls_outside_async_code_and_async_sleeps() {
    let source = "use tokio::time::sleep;
async fn offload(runtime: &Runtime) {
    tokio::task::spawn_blocking(|| std::thread::sleep(PAUSE)).await;
    let later = || runtime.block_on(job());
    tokio::time::sleep(PAUSE).await;
    sleep(PAUSE).await;
    scheduler::sleep(PAUSE).await;
    fn helper() {
        std::thread::sleep(PAUSE);
    }
    // DECISION: D001
    std::thread::sleep(PAUSE);
}
fn serve(runtime: &Runtime) {
    runtime.block_on(listen());
    std::thread::sleep(PAUSE);
}
#[cfg(test)]
mod tests {
    async fn check() {
        std::thread::sleep(PAUSE);
    }
}
";
    assert_eq!(findings(BLOCKING, source), []);
}

#[test]
fn rust_flags_std_fs_and_net_calls_not_awaited_inside_async_code() {
    let source = "async fn load(path: &Path) -> Result<Config, Error> {
    let text = std::fs::read_to_string(path)?;
    fs::write(path, &text)?;
    let stream = std::net::TcpStream::connect(ADDRESS)?;
    let listener = net::TcpListener::bind(ADDRESS)?;
    let task = async {
        fs::create_dir_all(path)
    };
    parse(&text)
}
";
    let expected = [
        (2, "fs::read_to_string"),
        (3, "fs::write"),
        (4, "net::TcpStream::connect"),
        (5, "net::TcpListener::bind"),
        (7, "fs::create_dir_all"),
    ]
    .map(|(line, symbol)| (line, symbol.to_string()));
    assert_eq!(findings(BLOCKING, source), expected);
}

#[test]
fn rust_accepts_awaited_or_other_io_and_std_io_outside_async_code() {
    let source = "async fn load(path: &Path) -> Result<String, Error> {
    let text = tokio::fs::read_to_string(path).await?;
    let again = fs::read_to_string(path).await?;
    let socket = UdpSocket::bind(ADDRESS)?;
    let content = tokio::task::spawn_blocking(move || std::fs::read(path)).await;
    let other = myfs::read(path);
    // DECISION: D001
    std::fs::write(path, &text)?;
    Ok(text)
}
fn sync(path: &Path) -> Result<String, Error> {
    std::fs::read_to_string(path)
}
#[cfg(test)]
mod tests {
    async fn check(path: &Path) {
        std::fs::read(path).unwrap();
    }
}
";
    assert_eq!(findings(BLOCKING, source), []);
}
