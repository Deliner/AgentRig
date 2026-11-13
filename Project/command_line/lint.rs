use agentrig::shared::console;

fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let code = match agentrig::structural_lint::reporting::cli::standalone(args) {
        Ok(code) => code,
        Err(error) => {
            console::stderr(format!("{error:#}"));
            2
        }
    };
    std::process::exit(code);
}
