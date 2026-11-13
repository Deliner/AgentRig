pub fn forwarded(args: &[String]) -> &[String] {
    args.strip_prefix(&["--".to_owned()]).unwrap_or(args)
}
