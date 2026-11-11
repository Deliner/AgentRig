use super::measured;
use crate::structural_lint::rules::ownership::Ownership;

fn found(source: &str, kind: Ownership) -> Vec<String> {
    measured(source, &[], kind)
        .into_iter()
        .map(|(line, symbol)| format!("{line}:{symbol}"))
        .collect()
}

#[test]
fn rust_flags_an_atomic_store_of_its_own_load_outside_test_code() {
    let source = "use std::sync::atomic::{AtomicU64, Ordering};
fn bump(hits: &AtomicU64, other: &AtomicU64) {
    hits.store(hits.load(Ordering::SeqCst) + 1, Ordering::SeqCst);
    hits.store(other.load(Ordering::SeqCst) + 1, Ordering::SeqCst);
    hits.store(1, Ordering::SeqCst);
    hits.fetch_add(1, Ordering::SeqCst);
    // DECISION: D001
    hits.store(hits.load(Ordering::SeqCst) + 2, Ordering::SeqCst);
}
impl Counter {
    fn double(&self) {
        self.count
            .store(self.count.load(Ordering::Relaxed).wrapping_mul(2), Ordering::Relaxed);
    }
}
#[cfg(test)]
mod tests {
    fn check(hits: &AtomicU64) {
        hits.store(hits.load(Ordering::SeqCst) + 1, Ordering::SeqCst);
    }
}
";
    assert_eq!(
        found(source, Ownership::LoadThenStore),
        ["3:hits", "12:self.count"]
    );
}

#[test]
fn rust_flags_a_wait_with_a_guard_and_no_loop_in_its_own_function() {
    let source = "use std::sync::{Condvar, Mutex};
fn ready(state: &Mutex<bool>, cv: &Condvar, child: &mut Child, barrier: &Barrier) {
    let guard = state.lock().unwrap();
    let guard = cv.wait(guard).unwrap();
    let (guard, _) = cv.wait_timeout(guard, PAUSE).unwrap();
    while !*guard {
        guard = cv.wait(guard).unwrap();
    }
    loop {
        let (next, _) = cv.wait_timeout(guard, PAUSE).unwrap();
        spawn(move || cv.wait(next));
    }
    let guard = cv.wait_while(guard, |done| !*done).unwrap();
    child.wait();
    barrier.wait();
    // DECISION: D001
    let guard = cv.wait(guard).unwrap();
}
#[cfg(test)]
mod tests {
    fn check(cv: &Condvar, guard: Guard) {
        let guard = cv.wait(guard).unwrap();
    }
}
";
    assert_eq!(
        found(source, Ownership::UnloopedWait),
        ["4:wait", "5:wait_timeout", "11:wait"]
    );
}

#[test]
fn rust_flags_a_listed_process_setting_changed_outside_main() {
    let source = "use std::{env, panic};
fn configure(dir: &Path) {
    unsafe { std::env::set_var(\"MODE\", \"fast\") };
    unsafe { env::remove_var(\"SLOW\") };
    env::set_current_dir(dir).unwrap();
    panic::set_hook(Box::new(|_| {}));
    log::set_logger(&LOGGER).unwrap();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    Command::new(\"make\").env(\"MODE\", \"fast\").current_dir(dir);
    set_var(\"MODE\", \"fast\");
    config::set_var(\"MODE\", \"fast\");
    // DECISION: D001
    env::set_current_dir(dir).unwrap();
}
fn main() {
    unsafe { env::set_var(\"MODE\", \"fast\") };
}
";
    assert_eq!(
        found(source, Ownership::ProcessSetting),
        [
            "3:set_var",
            "4:remove_var",
            "5:set_current_dir",
            "6:set_hook",
            "7:set_logger",
            "8:set_global_default",
        ]
    );
}

#[test]
fn rust_flags_a_process_setting_changed_in_test_code() {
    let source = "#[test]
fn reads_mode() {
    unsafe { env::set_var(\"MODE\", \"test\") };
}
#[cfg(test)]
mod tests {
    fn reset() {
        unsafe { env::remove_var(\"MODE\") };
    }
}
";
    assert_eq!(
        found(source, Ownership::ProcessSetting),
        ["3:set_var", "8:remove_var"]
    );
}
