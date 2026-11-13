use super::measured;
use crate::structural_lint::rule_identities::control_flow::ControlFlow;

#[test]
fn rust_flags_weak_exchanges_and_thread_parks_outside_a_loop_of_their_scope() {
    let source = "fn claim(flag: &AtomicBool) -> bool {
    let won = flag.compare_exchange_weak(false, true, SeqCst, Relaxed).is_ok();
    std::thread::park();
    thread::park_timeout(WAIT);
    loop {
        let later = || flag.compare_exchange_weak(false, true, SeqCst, Relaxed);
    }
    won
}
";
    assert_eq!(
        measured(source, ControlFlow::UnloopedRetry),
        [
            (2, "compare_exchange_weak".into()),
            (3, "park".into()),
            (4, "park_timeout".into()),
            (6, "compare_exchange_weak".into()),
        ]
    );
}

#[test]
fn rust_skips_looped_strong_bare_method_justified_and_test_retries() {
    let source = "fn claim(flag: &AtomicBool, worker: &Worker) {
    while flag.compare_exchange_weak(false, true, SeqCst, Relaxed).is_err() {}
    loop {
        let result = flag.compare_exchange(false, true, SeqCst, Relaxed);
        thread::park();
    }
    for _ in 0..3 {
        std::thread::park_timeout(WAIT);
    }
    park();
    worker.park();
    // DECISION: D001
    thread::park();
}
#[cfg(test)]
mod tests {
    fn check(flag: &AtomicBool) {
        flag.compare_exchange_weak(false, true, SeqCst, Relaxed);
    }
}
";
    assert!(measured(source, ControlFlow::UnloopedRetry).is_empty());
}
