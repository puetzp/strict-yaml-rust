//! Provides a [`serde`] implementation.
//!
//! The functions in this module can be used to serialize data
//! structures to StrictYAML and deserialize a StrictYAML string
//! back into a data structure.
//!
//! It provides an alternative to reading StrictYAML into a instance
//! of [`StrictYaml`](enum@crate::StrictYaml) and then possibly having to
//! convert it into a stronger type.

pub mod de;
pub mod error;

pub use de::from_str;
pub use de::from_str_many;
