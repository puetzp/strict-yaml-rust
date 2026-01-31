use crate::{
    parser::{Event, Parser},
    serde::error::Error,
    strict_yaml::StrictYaml,
};
use serde::de::{
    Deserialize, DeserializeSeed, EnumAccess, Error as SerdeError, IntoDeserializer, MapAccess,
    SeqAccess, Unexpected, VariantAccess, Visitor,
};
use std::str::{Chars, FromStr};

/// Deserialize an instance of type T from [`StrictYaml`](enum@crate::StrictYaml).
///
/// ```
/// use strict_yaml_rust::{StrictYaml, strict_yaml::Hash, serde::from_strict_yaml};
///
/// let yaml = StrictYaml::Array(
///     vec![
///         StrictYaml::String("1".into()),
///         StrictYaml::String("2".into()),
///         StrictYaml::String("3".into())
///     ]
/// );
///
/// assert_eq!(vec![1, 2, 3], from_strict_yaml::<Vec<u16>>(yaml).unwrap());
/// ```
pub fn from_strict_yaml<'a, T>(yaml: StrictYaml) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    T::deserialize(yaml)
}

pub struct Deserializer<'de> {
    parser: Parser<Chars<'de>>,
    many: bool,
    is_root: Option<bool>,
}

impl<'de> Deserializer<'de> {
    fn new(input: &'de str, many: bool) -> Self {
        Deserializer {
            parser: Parser::new(input.chars()),
            many,
            is_root: None,
        }
    }

    pub fn from_str_many(input: &'de str) -> Self {
        Deserializer::new(input, true)
    }

    pub fn from_str(input: &'de str) -> Self {
        Deserializer::new(input, false)
    }
}

/// Deserialize multiple StrictYAML documents from the same stream into an
/// instance of a container `T`.
///
/// The function serves as a hint to the deserializer to expect a
/// multi-document StrictYAML stream and process it accordingly. The hint from
/// the user is necessary because StrictYAML is not self-describing to the extent
/// that JSON is when it comes to mapping StrictYaml to the `serde` data model.
/// The deserializer needs a way to tell the difference between the
/// following cases when it calls [`serde::Deserializer::deserialize_seq`]:
///
/// ```yaml
/// ---
/// some: example
/// data: 100
/// ---
/// some: example
/// data: 200
/// ---
/// some: example
/// data: 300
/// ```
///
/// ```yaml
/// ---
/// - some: example
///   data: 100
/// - some: example
///   data: 200
/// - some: example
///   data: 300
/// ```
///
/// [`from_str_many`] handles the former case while the latter is the "default
/// mode" when calling [`from_str`].
///
/// # Examples
///
/// As described above the [`from_str_many`] function deserializes a YAML stream containing
/// multiple documents to a container data structure that implements
/// [`serde::Deserialize`] (e.g. [`Vec`]).
///
///
/// ```rust
/// use strict_yaml_rust::serde::from_str_many;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Deployment {
///   kind: String,
///   spec: Spec
/// }
///
/// #[derive(Deserialize)]
/// struct Spec {
///   replicas: u16,
///   name: String
/// }
///
/// let yaml = r#"
/// ---
/// kind: deployment
/// spec:
///   replicas: 5
///   name: "nginx"
/// ---
/// kind: container
/// spec:
///   replicas: 1
///   name: "redis"
/// ---
/// kind: deployment
/// spec:
///   replicas: 3
///   name: "webapp"
/// ...
/// "#;
///
/// let deployments: Vec<Deployment> = from_str_many(yaml).unwrap();
///
/// assert!(deployments.len() == 3);
/// assert!(deployments.first().is_some_and(|d| d.spec.name == "nginx".to_string()));
/// ```
pub fn from_str_many<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str_many(s);

    T::deserialize(&mut deserializer)
}

/// Deserialize a YAML document into an instance of type `T`.
///
/// # Examples
///
/// The [`from_str`] function deserializes a YAML document to a data structure
/// that implements [`serde::Deserialize`].
///
///
/// ```rust
/// use strict_yaml_rust::serde::from_str;
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Deployment {
///   kind: String,
///   spec: Spec
/// }
///
/// #[derive(Deserialize)]
/// struct Spec {
///   replicas: u16,
///   name: String
/// }
///
/// let yaml = r#"
/// ---
/// kind: deployment
/// spec:
///   replicas: 5
///   name: "nginx"
/// ...
/// "#;
///
/// let deployment: Deployment = from_str(yaml).unwrap();
///
/// assert_eq!(deployment.spec.name, "nginx".to_string());
/// ```
pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);

    let (ev, mark) = deserializer.parser.next()?;

    if ev != Event::StreamStart {
        return Err(Error::from_event(ev, mark, "the start of the stream"));
    }

    let (ev, _mark) = deserializer.parser.peek()?;

    if *ev == Event::DocumentStart {
        deserializer.parser.next()?;
    }

    let res = T::deserialize(&mut deserializer)?;

    let (ev, _mark) = deserializer.parser.peek()?;

    if *ev == Event::DocumentEnd {
        deserializer.parser.next()?;
    }

    let (ev, mark) = deserializer.parser.next()?;

    if ev != Event::StreamEnd {
        return Err(Error::from_event(ev, mark, "the end of the stream"));
    }

    Ok(res)
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.peek()?;

        match ev {
            Event::Scalar(_, _, _) => self.deserialize_str(visitor),
            Event::SequenceStart(_) => self.deserialize_seq(visitor),
            Event::MappingStart(_) => self.deserialize_map(visitor),
            _ => {
                return Err(Error::from_event(
                    ev.clone(),
                    mark.clone(),
                    "a sequence, map or scalar",
                ))
            }
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let boolean = bool::from_str(&value)
                    .map_err(|_| Error::invalid_value(Unexpected::Str(&value), &"a boolean"))
                    .map_err(|err| err + mark)?;

                visitor.visit_bool(boolean).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i8::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"an 8-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i8(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i16::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 16-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i16(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i32::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 32-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i32(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i64::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 64-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i64(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_i128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = i128::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"an 128-bit signed integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_i128(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u8::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"an 8-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u8(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u16::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 16-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u16(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u32::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 32-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u32(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u64::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a 64-bit unsigned integer")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u64(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_u128<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = u128::from_str_radix(&value, 10)
                    .map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(&value),
                            &"an 128-bit unsigned integer",
                        )
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_u128(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = f32::from_str(&value)
                    .map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(&value),
                            &"a 32-bit floating point number",
                        )
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_f32(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let num = f64::from_str(&value)
                    .map_err(|_| {
                        Error::invalid_value(
                            Unexpected::Str(&value),
                            &"a 64-bit floating point number",
                        )
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_f64(num).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                let c = char::from_str(&value)
                    .map_err(|_| {
                        Error::invalid_value(Unexpected::Str(&value), &"a single character")
                    })
                    .map_err(|err| err + mark)?;

                visitor.visit_char(c).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _style, _anchor_id) => {
                visitor.visit_string(value).map_err(|err: Error| err + mark)
            }
            _ => Err(Error::from_event(ev, mark, "a scalar")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        Err(Error::UnsupportedType("bytes"))
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, _mark) = self.parser.peek()?;

        let is_some = match ev {
            Event::MappingStart(_) | Event::SequenceStart(_) | Event::Scalar(_, _, _) => true,
            _ => false,
        };

        if is_some {
            visitor.visit_some(self)
        } else {
            visitor.visit_none()
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, _mark) = self.parser.peek()?;

        if matches!(ev, Event::Scalar(value, _style, _anchor_id) if value.is_empty()) {
            let _ = self.parser.next()?;
        }

        visitor.visit_unit()
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let mut is_root = false;

        if self.many && self.is_root.is_none() {
            self.is_root = Some(true);
            is_root = true;
        }

        let (ev, mark) = self.parser.next()?;

        let value = match ev {
            Event::SequenceStart(_) => visitor
                .visit_seq(ArrayAccess::new(self, is_root))
                .map_err(|err: Error| err + mark)?,
            Event::StreamStart if self.many => visitor
                .visit_seq(ArrayAccess::new(self, is_root))
                .map_err(|err: Error| err + mark)?,
            _ => {
                if self.many && is_root {
                    return Err(Error::from_event(ev, mark, "the start of the stream"));
                } else {
                    return Err(Error::from_event(ev, mark, "the start of a sequence"));
                }
            }
        };

        if self.many && is_root {
            let (ev, mark) = self.parser.next()?;

            if ev != Event::StreamEnd {
                self.is_root = Some(false);
                return Err(Error::from_event(ev, mark, "the end of the stream"));
            }
        } else {
            let (ev, mark) = self.parser.next()?;

            if ev != Event::SequenceEnd {
                return Err(Error::from_event(ev, mark, "the end of a sequence"));
            }
        }

        Ok(value)
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        let value = match ev {
            Event::MappingStart(_) => visitor
                .visit_map(HashAccess::new(self))
                .map_err(|err: Error| err + mark)?,
            _ => return Err(Error::from_event(ev, mark, "the start of a mapping")),
        };

        let (ev, _mark) = self.parser.next()?;

        if ev != Event::MappingEnd {
            Err(Error::from_event(ev, mark, "the end of a mapping"))
        } else {
            Ok(value)
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (ev, mark) = self.parser.next()?;

        match ev {
            Event::Scalar(value, _, _) => visitor
                .visit_enum(value.into_deserializer())
                .map_err(|err: Error| err + mark),
            _ => {
                let v = visitor
                    .visit_enum(Enum::new(self))
                    .map_err(|err: Error| err + mark)?;

                let (ev, mark) = self.parser.next()?;

                if ev != Event::MappingEnd {
                    Err(Error::from_event(ev, mark, "the end of a mapping"))
                } else {
                    Ok(v)
                }
            }
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct ArrayAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
    is_root: bool,
}

impl<'a, 'de> ArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>, is_root: bool) -> Self {
        Self { de, is_root }
    }
}

impl<'de, 'a> SeqAccess<'de> for ArrayAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        let (ev, _mark) = self.de.parser.peek()?;

        if self.de.many && self.is_root {
            match ev {
                Event::StreamEnd => Ok(None),
                Event::DocumentStart => {
                    let _ = self.de.parser.next()?;

                    let v = seed.deserialize(&mut *self.de).map(Some);

                    let (ev, _mark) = self.de.parser.peek()?;

                    if *ev == Event::DocumentEnd {
                        let _ = self.de.parser.next()?;
                    }

                    v
                }
                _ => unreachable!(),
            }
        } else {
            match ev {
                Event::SequenceEnd => Ok(None),
                _ => seed.deserialize(&mut *self.de).map(Some),
            }
        }
    }
}

struct HashAccess<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> HashAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> MapAccess<'de> for HashAccess<'a, 'de> {
    type Error = Error;

    fn next_key_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        let (ev, _mark) = self.de.parser.peek()?;

        match ev {
            Event::MappingEnd => Ok(None),
            _ => seed.deserialize(&mut *self.de).map(Some),
        }
    }

    fn next_value_seed<T>(&mut self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(&mut *self.de)
    }
}

struct Enum<'a, 'de: 'a> {
    de: &'a mut Deserializer<'de>,
}

impl<'a, 'de> Enum<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> EnumAccess<'de> for Enum<'a, 'de> {
    type Error = Error;
    type Variant = Self;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let v = seed.deserialize(&mut *self.de)?;
        Ok((v, self))
    }
}

impl<'de, 'a> VariantAccess<'de> for Enum<'a, 'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        seed.deserialize(self.de)
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_seq(self.de, visitor)
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        serde::de::Deserializer::deserialize_map(self.de, visitor)
    }
}

#[cfg(test)]
mod test {
    use super::{from_str, from_str_many};
    use serde::Deserialize;

    #[test]
    fn test_primitives() {
        let input = r#"
true
"#;

        assert_eq!(true, from_str(input).unwrap());

        let input = r#"
"false"
"#;

        assert_eq!(false, from_str(input).unwrap());

        let input = r#"
foobar
"#;

        assert_eq!("foobar".to_string(), from_str::<String>(input).unwrap());

        let input = r#"
'foobar'
"#;

        assert_eq!("foobar".to_string(), from_str::<String>(input).unwrap());

        let input = r#"
78
"#;

        assert_eq!(78, from_str(input).unwrap());

        let input = r#"
-78
"#;

        assert_eq!(-78, from_str(input).unwrap());

        let input = r#"
'-78'
"#;

        assert_eq!(-78, from_str(input).unwrap());

        let input = r#"
"-78"
"#;

        assert_eq!(-78, from_str(input).unwrap());

        let input = r#"
7.8
"#;

        assert_eq!(7.8, from_str(input).unwrap());

        let input = r#"
-7.8
"#;

        assert_eq!(-7.8, from_str(input).unwrap());

        let input = r#"
"%"
"#;

        assert_eq!('%', from_str(input).unwrap());
    }

    #[test]
    fn test_option() {
        let input = r#"
foobar
"#;

        assert_eq!(
            Some("foobar".to_string()),
            from_str::<Option<String>>(input).unwrap()
        );
    }

    #[test]
    fn test_unit() {
        assert_eq!((), from_str("").unwrap());

        let input = r#"
---
"#;

        assert_eq!((), from_str(input).unwrap());
    }

    #[test]
    fn test_unit_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test;

        assert_eq!(Test, from_str("").unwrap());

        let input = r#"
---
"#;

        assert_eq!(Test, from_str(input).unwrap());
    }

    #[test]
    fn test_newtype_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test(String);

        let input = r#"
foobar
"#;

        assert_eq!(Test("foobar".to_string()), from_str(input).unwrap());
    }

    #[test]
    fn test_seq() {
        let input = r#"
- foo
- bar
- foobar
"#;

        let expected = vec!["foo", "bar", "foobar"];

        assert_eq!(expected, from_str::<Vec<String>>(input).unwrap());
    }

    #[test]
    fn test_tuple() {
        let input = r#"
- foobar
- false
- 8
"#;

        let expected = ("foobar".to_string(), false, 8);

        assert_eq!(expected, from_str::<(String, bool, u8)>(input).unwrap());
    }

    #[test]
    fn test_tuple_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test(String, bool, u8);

        let input = r#"
- foobar
- false
- 8
"#;

        let expected = Test("foobar".to_string(), false, 8);

        assert_eq!(expected, from_str(input).unwrap());
    }

    #[test]
    fn test_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            a: bool,
            b: String,
            c: i64,
            d: f64,
        }

        let input = r#"
a: true
b: |
  foo
  bar
c: -56
d: 5
"#;

        let expected = Test {
            a: true,
            b: "foo\nbar\n".to_string(),
            c: -56,
            d: 5.0,
        };

        assert_eq!(expected, from_str(input).unwrap());
    }

    #[test]
    fn test_complex_struct() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct Test {
            a: bool,
            b: Vec<Item>,
            c: (u8, u8, bool),
            d: Sub,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Item {
            foo: String,
            bar: f64,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        struct Sub {
            x: bool,
            y: String,
            z: i64,
        }

        let input = r#"
c:
  - 10
  - 12
  - false
b:
  - foo: some value
    bar: 100.1234
  - foo: other value
    bar: 101.1234
  - foo: final value
    bar: 102.1234
a: false
d:
  z: 6
  x: false
  y: |
    foo
    bar
"#;

        let expected = Test {
            a: false,
            b: vec![
                Item {
                    foo: "some value".to_string(),
                    bar: 100.1234,
                },
                Item {
                    foo: "other value".to_string(),
                    bar: 101.1234,
                },
                Item {
                    foo: "final value".to_string(),
                    bar: 102.1234,
                },
            ],
            c: (10, 12, false),
            d: Sub {
                z: 6,
                x: false,
                y: "foo\nbar\n".to_string(),
            },
        };

        assert_eq!(expected, from_str(input).unwrap());
    }

    #[test]
    fn test_enum_unit_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            First,
            Second,
        }

        let input = r#"
First
"#;

        assert_eq!(Test::First, from_str(input).unwrap());

        let input = r#"
---
First
---
Second
"#;

        let expected = vec![Test::First, Test::Second];

        assert_eq!(expected, from_str_many::<Vec<Test>>(input).unwrap());
    }

    #[test]
    fn test_enum_newtype_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            First(String),
            Second(bool),
        }

        let input = r#"
First: foobar
"#;

        assert_eq!(Test::First("foobar".to_string()), from_str(input).unwrap());
    }

    #[test]
    fn test_enum_tuple_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            Foo(u8, bool, String),
            Bar,
        }

        let input = r#"
---
Foo:
  - 10
  - true
  - bar
---
Bar
"#;

        let expected = vec![Test::Foo(10, true, "bar".to_string()), Test::Bar];

        assert_eq!(expected, from_str_many::<Vec<Test>>(input).unwrap());
    }

    #[test]
    fn test_enum_struct_variant() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "lowercase")]
        enum Test {
            Foo { a: u8, b: bool, c: String },
            Bar,
        }

        let input = r#"
---
foo:
  b: true
  c: bar
  a: 10
---
bar
"#;

        let expected = vec![
            Test::Foo {
                a: 10,
                b: true,
                c: "bar".to_string(),
            },
            Test::Bar,
        ];

        assert_eq!(expected, from_str_many::<Vec<Test>>(input).unwrap());
    }

    #[test]
    fn test_multiple_documents() {
        let input = r#"
---
foobar
---
barfoo
---
end
"#;

        let expected = vec![
            "foobar".to_string(),
            "barfoo".to_string(),
            "end".to_string(),
        ];

        assert_eq!(expected, from_str_many::<Vec<String>>(input).unwrap());

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Test {
            a: String,
            b: usize,
            c: bool,
        }

        let input = r#"
---
a: foo
b: 50
c: true
---
b: 10
a: bar
c: false
...
---
c: false
b: 20
a: end
...
"#;

        let expected = vec![
            Test {
                a: "foo".to_string(),
                b: 50,
                c: true,
            },
            Test {
                a: "bar".to_string(),
                b: 10,
                c: false,
            },
            Test {
                a: "end".to_string(),
                b: 20,
                c: false,
            },
        ];

        assert_eq!(expected, from_str_many::<Vec<Test>>(input).unwrap());
    }

    #[test]
    fn test_deeply_nested() {
        use std::{collections::HashMap, default::Default};

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Deployment {
            #[serde(rename = "apiVersion")]
            api_version: String,
            kind: Kind,
            metadata: Metadata,
            spec: DeploymentSpec,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        enum Kind {
            Deployment,
            StatefulSet,
            DaemonSet,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Metadata {
            #[serde(default)]
            name: Option<String>,
            labels: HashMap<String, String>,
            #[serde(default)]
            iteration: usize,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct DeploymentSpec {
            #[serde(default)]
            replicas: usize,
            selector: Selector,
            template: Template,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        enum Selector {
            #[serde(rename = "matchLabels")]
            ByLabel(HashMap<String, String>),
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Template {
            metadata: Metadata,
            spec: ContainerSpec,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct ContainerSpec {
            containers: Vec<Container>,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(deny_unknown_fields)]
        struct Container {
            name: String,
            image: String,
            ports: Vec<Port>,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        enum Port {
            #[serde(rename = "containerPort")]
            ContainerPort(usize),
        }

        let input = r#"
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nginx-deployment
  labels:
    app: nginx
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nginx
  template:
    metadata:
      labels:
        app: nginx
    spec:
      containers:
      - name: nginx
        image: nginx:1.14.2
        ports:
        - containerPort: 80
"#;

        let expected = Deployment {
            api_version: "apps/v1".to_string(),
            kind: Kind::Deployment,
            metadata: Metadata {
                name: Some("nginx-deployment".to_string()),
                labels: HashMap::from([("app".to_string(), "nginx".to_string())]),
                iteration: Default::default(),
            },
            spec: DeploymentSpec {
                replicas: 3,
                selector: Selector::ByLabel(HashMap::from([(
                    "app".to_string(),
                    "nginx".to_string(),
                )])),
                template: Template {
                    metadata: Metadata {
                        name: None,
                        labels: HashMap::from([("app".to_string(), "nginx".to_string())]),
                        iteration: Default::default(),
                    },
                    spec: ContainerSpec {
                        containers: vec![Container {
                            name: "nginx".to_string(),
                            image: "nginx:1.14.2".to_string(),
                            ports: vec![Port::ContainerPort(80)],
                        }],
                    },
                },
            },
        };

        assert_eq!(expected, from_str(input).unwrap());
    }
}
