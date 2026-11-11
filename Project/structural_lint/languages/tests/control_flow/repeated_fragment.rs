use super::measured;
use crate::structural_lint::rules::control_flow::ControlFlow;

#[test]
fn rust_flags_the_second_use_of_an_expression_fragment_in_one_transcriber() {
    let source = "macro_rules! positive {
    ($x:expr) => {
        if $x > 0 { $x } else { 0 }
    };
}
macro_rules! pair {
    ($y:expr_2021) => {
        ($y,
         $y)
    };
}
macro_rules! listed {
    ($($item:expr),*) => {
        let first = vec![$($item),*];
        let second = vec![$($item),*];
    };
}
";
    assert_eq!(
        measured(source, ControlFlow::RepeatedFragment),
        [
            (3, "$x".to_string()),
            (9, "$y".to_string()),
            (15, "$item".to_string()),
        ]
    );
}

#[test]
fn rust_skips_single_stringified_other_fragment_justified_and_test_uses() {
    let source = "macro_rules! once {
    ($x:expr, $name:ident) => {
        println!(\"{} = {}\", stringify!($x), $x);
        let $name = $name;
    };
}
macro_rules! justified {
    ($x:expr) => {
        let a = $x;
        // DECISION: D001
        let b = $x;
    };
}
#[cfg(test)]
mod tests {
    macro_rules! check {
        ($x:expr) => { $x + $x };
    }
}
";
    assert!(measured(source, ControlFlow::RepeatedFragment).is_empty());
}
