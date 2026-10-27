use super::take_option;

fn arguments(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn extraction_consumes_one_option_and_preserves_remaining_order() {
    let mut args = arguments(&["run", "--root", "first", "task", "--root", "second"]);
    assert_eq!(
        take_option(&mut args, "--root").unwrap().as_deref(),
        Some("first")
    );
    assert_eq!(args, ["run", "task", "--root", "second"]);
    assert_eq!(
        take_option(&mut args, "--root").unwrap().as_deref(),
        Some("second")
    );
    assert_eq!(args, ["run", "task"]);
    assert!(take_option(&mut args, "--root").unwrap().is_none());
    assert_eq!(args, ["run", "task"]);
}

#[test]
fn literal_separator_stops_option_lookup_and_single_dash_values_are_valid() {
    let mut args = arguments(&["--root", "-value", "--", "--root", "literal"]);
    assert_eq!(
        take_option(&mut args, "--root").unwrap().as_deref(),
        Some("-value")
    );
    assert!(take_option(&mut args, "--root").unwrap().is_none());
    assert_eq!(args, ["--", "--root", "literal"]);
    let mut empty = arguments(&["--root", ""]);
    assert_eq!(
        take_option(&mut empty, "--root").unwrap().as_deref(),
        Some("")
    );
    assert!(empty.is_empty());
}

#[test]
fn missing_value_removes_only_the_option_and_preserves_the_following_flag() {
    for suffix in [&[][..], &["--other"][..], &["--", "literal"][..]] {
        let mut args = arguments(&["--root"]);
        args.extend(arguments(suffix));
        let error = take_option(&mut args, "--root").unwrap_err();
        assert_eq!(error.to_string(), "--root requires a value");
        assert_eq!(args, arguments(suffix));
    }
}
