fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if let Err(error) = review_runner::cli::run(&args) {
        eprintln!("{error:#}");
        std::process::exit(2);
    }
}
