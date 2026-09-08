pub mod architecture;
pub(super) mod descriptor;
pub mod kind;

pub use super::catalog::{
    ALL, catalog, describe, example, extensions, support, supports_path, syntax,
    validate_extensions, validate_includes,
};
pub use descriptor::{Descriptor, Parameters};
pub use kind::Kind;
