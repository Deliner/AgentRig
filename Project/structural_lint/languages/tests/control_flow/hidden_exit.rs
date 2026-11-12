use super::measured;
use crate::structural_lint::rules::control_flow::ControlFlow;

#[test]
fn rust_flags_returns_propagation_and_exits_of_the_caller_loop_in_a_transcriber() {
    let source = "macro_rules! need {
    ($opt:expr) => {
        match $opt {
            Some(value) => value,
            None => return Err(Error::Missing),
        }
    };
}
macro_rules! unwrap {
    ($result:expr) => { $result? };
}
macro_rules! skip {
    ($cond:expr) => {
        if $cond {
            continue;
        }
    };
}
macro_rules! retry_once {
    ($value:expr) => { return $value };
}
macro_rules! Bail {
    ($value:expr) => { return $value };
}
";
    assert_eq!(
        measured(source, ControlFlow::HiddenExit),
        [
            (5, "return".to_string()),
            (10, "?".to_string()),
            (15, "continue".to_string()),
            (20, "return".to_string()),
            (23, "return".to_string()),
        ]
    );
}

#[test]
fn rust_skips_transcribers_of_macros_named_by_an_exit_word() {
    let source = "macro_rules! try_get {
    ($map:expr, $key:expr) => { $map.get($key)? };
}
macro_rules! ensure {
    ($cond:expr) => { if !$cond { return Err(Error::Failed); } };
}
macro_rules! bail_if_none {
    ($opt:expr) => { match $opt { Some(value) => value, None => return None } };
}
";
    assert!(measured(source, ControlFlow::HiddenExit).is_empty());
}

#[test]
fn rust_skips_own_loops_deferred_bodies_unparsed_justified_and_test_transcribers() {
    let source = "macro_rules! each {
    ($($item:expr),*) => {
        $(
            for value in $item {
                if value { break; }
            }
        )*
    };
}
macro_rules! deferred {
    ($value:expr) => {
        let check = || -> Option<u8> { $value?; None };
        let task = async { return 1; };
        fn helper() -> u8 { return 2; }
    };
}
macro_rules! broken {
    ($x:expr) => { => return $x };
}
macro_rules! give_up {
    ($error:expr) => {
        // DECISION: D001
        return Err($error.into())
    };
}
#[cfg(test)]
mod tests {
    macro_rules! check {
        ($x:expr) => { return $x };
    }
}
";
    assert!(measured(source, ControlFlow::HiddenExit).is_empty());
}
