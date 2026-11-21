use crate::structural_lint::languages::tests::{line_actuals, symbol_actuals};
use crate::structural_lint::rule_identities::{ControlFlow, Kind};

// DECISION: D061
fn nesting_depths(path: &str, source: &str) -> Vec<(String, usize)> {
    symbol_actuals(Kind::ControlFlow(ControlFlow::NestingDepth), path, source)
}

#[test]
fn rust_nesting_counts_blocks_without_else_if_or_nested_items() {
    let source = "fn flat() {
    let value = 1;
}
fn nested() {
    for value in items {
        if value > 0 {
            while value > 0 {
                work();
            }
        }
    }
}
fn ladder(choice: i32) {
    if choice == 0 {
    } else if choice == 1 {
    } else {
    }
}
fn outer() {
    let closure = |value: i32| {
        if value > 0 {
            work();
        }
    };
}
";
    assert_eq!(
        nesting_depths("source.rs", source),
        [
            ("<anonymous>".into(), 1),
            ("flat".into(), 0),
            ("ladder".into(), 1),
            ("nested".into(), 3),
            ("outer".into(), 0),
        ]
    );
}

#[test]
fn python_nesting_counts_blocks_without_elif_or_nested_functions() {
    let source = "def flat():
    value = 1
def nested():
    for value in items:
        if value > 0:
            while value > 0:
                work()
def ladder(choice):
    if choice == 0:
        pass
    elif choice == 1:
        pass
    else:
        pass
def outer():
    def inner():
        if ready:
            work()
";
    assert_eq!(
        nesting_depths("source.py", source),
        [
            ("flat".into(), 0),
            ("inner".into(), 1),
            ("ladder".into(), 1),
            ("nested".into(), 3),
            ("outer".into(), 0),
        ]
    );
}

// DECISION: D063
fn branch_budgets(path: &str, source: &str) -> Vec<(String, usize)> {
    symbol_actuals(Kind::ControlFlow(ControlFlow::BranchBudget), path, source)
}

#[test]
fn rust_counts_ifs_and_matches_within_each_function() {
    let source = "fn flat() {
    let value = 1;
}
fn ladder(choice: i32) {
    if choice == 0 {
    } else if choice == 1 {
    } else {
    }
}
fn dispatch(choice: i32) -> i32 {
    match choice {
        0 => 10,
        1 => 20,
        _ => 30,
    }
}
fn outer(choice: i32) {
    let closure = |value: i32| {
        if value > 0 {
            work();
        }
    };
    if choice > 0 {
        work();
    }
}
";
    assert_eq!(
        branch_budgets("source.rs", source),
        [
            ("<anonymous>".into(), 1),
            ("dispatch".into(), 1),
            ("flat".into(), 0),
            ("ladder".into(), 2),
            ("outer".into(), 1),
        ]
    );
}

#[test]
fn python_counts_ifs_elifs_ternaries_and_matches_within_each_function() {
    let source = "def flat():
    value = 1
def ladder(choice):
    if choice == 0:
        pass
    elif choice == 1:
        pass
    else:
        pass
def dispatch(choice):
    match choice:
        case 0:
            return 10
        case _:
            return 20
def ternary(choice):
    return 1 if choice else 2
def outer(choice):
    def inner(value):
        if value:
            pass
    if choice:
        pass
";
    assert_eq!(
        branch_budgets("source.py", source),
        [
            ("dispatch".into(), 1),
            ("flat".into(), 0),
            ("inner".into(), 1),
            ("ladder".into(), 2),
            ("outer".into(), 1),
            ("ternary".into(), 1),
        ]
    );
}

// DECISION: D120
#[test]
fn rust_counts_operands_of_each_boolean_expression_wherever_it_stands() {
    let source = "fn ready() -> bool {
    let ready = active && !banned || admin && valid() && !maintenance;
    while !empty && budget > 0 && !cancelled && retries < 3 {}
    match value {
        Some(v) if v.a && v.b && v.c && v.d => {}
        _ => {}
    }
    let short = a && b && c;
    work(a || b);
    left == right && done
}
";
    assert_eq!(
        line_actuals(
            Kind::ControlFlow(ControlFlow::BooleanExpression),
            "source.rs",
            source
        ),
        [(2, 5), (3, 4), (5, 4), (8, 3), (9, 2), (10, 2)]
    );
}

#[test]
fn rust_groups_parentheses_and_measures_negations_let_bindings_and_macros_apart() {
    let source = "fn group() {
    let grouped = (a && b) || (c && d);
    let negated = x && !(a && b && c && d);
    if let Some(user) = session && user.active && !user.banned && admin {}
    if let Some(a) = left && let Some(b) = right && ready {}
    let call = f(a && b, c || d || e);
    assert!(a && b && c && d && e);
}
";
    assert_eq!(
        line_actuals(
            Kind::ControlFlow(ControlFlow::BooleanExpression),
            "source.rs",
            source
        ),
        [(2, 4), (3, 2), (3, 4), (4, 3), (5, 1), (6, 2), (6, 3)]
    );
}

#[test]
fn python_counts_and_or_operands_with_parentheses_and_not_as_one() {
    let source = "def ready():
    ready = active and not banned or admin and valid() and not maintenance
    while queue and budget > 0 and not cancelled and retries < 3:
        pass
    grouped = (a and b) or (c and d)
    negated = x and not (a and b and c and d)
    items = [x for x in values if x.a and x.b and x.c]
    return left == right and done
";
    assert_eq!(
        line_actuals(
            Kind::ControlFlow(ControlFlow::BooleanExpression),
            "source.py",
            source
        ),
        [(2, 5), (3, 4), (5, 4), (6, 2), (6, 4), (7, 3), (8, 2)]
    );
}
