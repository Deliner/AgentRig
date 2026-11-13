pub fn forwarded(args: &[String]) -> &[String] {
    args.trim_prefix(&["--".to_owned()])
}
