use super::measured;
use crate::structural_lint::rules::reference::Reference;

#[test]
fn rust_flags_thread_sleep_in_a_test_function_and_its_closures() {
    let source = "use std::thread::sleep;
#[test]
fn worker_finishes() {
    let worker = start_worker();
    thread::sleep(Duration::from_millis(200));
    std::thread::spawn(|| {
        std::thread::sleep(Duration::from_millis(5));
    });
    sleep(Duration::from_millis(1));
    assert!(worker.is_done());
}
#[tokio::test]
async fn polls() {
    thread::sleep(Duration::from_millis(10));
}
";
    assert_eq!(
        measured(source, Reference::SleepInTest),
        [
            (5, "sleep".to_string()),
            (7, "sleep".to_string()),
            (9, "sleep".to_string()),
            (14, "sleep".to_string()),
        ]
    );
}

#[test]
fn rust_skips_production_helpers_nested_functions_other_sleeps_and_justified_calls() {
    let source = "fn production() {
    thread::sleep(Duration::from_millis(5));
}
#[cfg(test)]
mod tests {
    fn helper() {
        thread::sleep(Duration::from_millis(5));
    }
    #[test]
    fn nested() {
        fn inner() {
            thread::sleep(Duration::from_millis(5));
        }
        tokio::time::sleep(Duration::from_millis(5));
        sleep(Duration::from_millis(5));
        // DECISION: D001
        thread::sleep(Duration::from_millis(5));
    }
}
";
    assert!(measured(source, Reference::SleepInTest).is_empty());
}
