mod macro_origins;
mod module_tree;
mod resolver;
#[cfg(test)]
mod tests;
mod use_aliases;

pub use resolver::Rust;
