// DECISION: D109
#![expect(clippy::print_stdout, clippy::print_stderr)]
use std::{
    fmt::Display,
    io::{self, Write},
};

pub fn stdout(line: impl Display) {
    println!("{line}");
}
pub fn stderr(line: impl Display) {
    eprintln!("{line}");
}
pub fn prompt(text: impl Display) -> io::Result<()> {
    print!("{text}");
    io::stdout().flush()
}
