//! Provides a [`serde`] implementation.
//!
//! The functions in this module can be used to serialize data
//! structures to StrictYAML and deserialize a data structure from
//! a StrictYAML string.
//!
//! Note that this API is distinct from parsing a StrictYAML string
//! to an instance of [`StrictYaml`](enum@crate::StrictYaml) and
//! then possibly having to convert it into a stronger type.

pub mod de;
pub mod error;
pub mod ser;

pub use de::from_str;
pub use de::from_str_many;
pub use de::from_strict_yaml;
pub use ser::to_strict_yaml;
