use super::findings;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_if_let_ok_without_else_alone_in_loops_in_let_chains_and_in_else_if_tails() {
    let source = "fn load(path: &Path, items: Vec<Result<u32, Error>>, cell: &Mutex<u32>, flag: bool) -> u32 {
    let mut total = 0;
    if let Ok(text) = fs::read_to_string(path) {
        total += text.len();
    }
    for item in items {
        if let Ok(value) = item {
            total += value;
        }
    }
    if let Ok(port) = env::var(\"PORT\")?.parse::<u16>() && flag {
        total += 1;
    }
    if flag {
        total += 1;
    } else if let Ok(guard) = cell.lock() {
        total += *guard;
    }
    if let Some(limit) = limit && let Result::Ok(value) = fetch(limit).await {
        total += value;
    }
    total
}
";
    assert_eq!(
        findings(Reference::UnhandledError, source),
        ["3:Ok", "7:Ok", "11:Ok", "16:Ok", "19:Ok"]
    );
}

#[test]
fn rust_keeps_busy_or_answering_errors_else_branches_other_forms_justified_and_test_code() {
    let source = "fn keep(cell: &Mutex<u32>, rx: &Receiver<u32>, sorted: &[u32], shared: Arc<State>) -> Result<u32> {
    if let Ok(guard) = cell.try_lock() {}
    if let Ok(value) = rx.recv_timeout(delay) {}
    if let Ok(message) = channel.recv().await {}
    if let Ok(item) = queue.try_recv().await? {}
    if let Ok(borrowed) = state.try_borrow_mut() {}
    if let Ok(index) = sorted.binary_search_by_key(&3, |item| *item) {}
    if let Ok(previous) = flag.compare_exchange(false, true, SeqCst, SeqCst) {}
    if let Ok(state) = Arc::try_unwrap(shared) {}
    if let Ok(text) = boxed.downcast::<String>() {}
    if let Ok(text) = fs::read_to_string(path) {
        show(text);
    } else {
        report();
    }
    if let Ok(text) = fs::read_to_string(path) {
    } else if ready {
    }
    while let Ok(value) = rx.recv() {}
    let Ok(value) = item.parse::<u32>() else {
        return Ok(0);
    };
    if let Some(value) = text.parse::<u32>().ok() {}
    if let Err(error) = save() {
        log(error);
    }
    // DECISION: D001
    if let Ok(text) = fs::read_to_string(path) {}
    Ok(0)
}
#[cfg(test)]
mod tests {
    fn probe() {
        if let Ok(text) = fs::read_to_string(\"x\") {}
    }
}
";
    assert!(findings(Reference::UnhandledError, source).is_empty());
}
