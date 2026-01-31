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
pub use ser::to_string;

#[cfg(test)]
mod test {
    use super::*;
    use crate::strict_yaml::StrictYaml;
    use serde::{Deserialize, Serialize};

    #[test]
    fn test_struct_de_ser() {
        #[derive(Debug, Deserialize, PartialEq, Serialize)]
        #[serde(deny_unknown_fields)]
        struct Test {
            a: String,
            b: usize,
            c: bool,
            d: u8,
        }

        let input = r#"---
a: foo
b: "50"
c: "true"
d: "2""#;

        let expected = Test {
            a: "foo".to_string(),
            b: 50,
            c: true,
            d: 2,
        };

        assert_eq!(expected, from_str::<Test>(input).unwrap());
        assert_eq!(input, to_string(&expected).unwrap());
    }

    #[test]
    fn test_complex_de_ser() {
        let input = r#"---
a:
  b:
    c: hello
  d: "{}"
e:
  - f
  - g
  - h: "[]"
  - a:
      - b
      - c
    d: e
a: b"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_nested_map_de_ser() {
        let input = r#"---
a:
  b:
    c:
      d:
        e: f"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }

    #[test]
    fn test_nested_array_de_ser() {
        let input = r#"---
a:
  - b
  - - c
    - d
    - - e
      - - f
      - - e"#;

        let yaml = from_str::<StrictYaml>(input).unwrap();

        assert_eq!(input, to_string(&yaml).unwrap());
    }
}
