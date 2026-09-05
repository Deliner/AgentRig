fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let code = match discipline_worker::lint::cli::standalone(args) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error:#}");
            2
        }
    };
    std::process::exit(code);
}
