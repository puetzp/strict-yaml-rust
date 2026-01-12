use crate::{
    parser::{Event, Parser},
    serde::error::Error,
};
use serde::de::{
    Deserialize, DeserializeSeed, Error as SerdeError, MapAccess, SeqAccess, Unexpected, Visitor,
};
use std::str::{Chars, FromStr};

pub struct Deserializer<'de> {
    parser: Parser<Chars<'de>>,
    many: bool,
}

impl<'de> Deserializer<'de> {
    fn new(input: &'de str, many: bool) -> Self {
        Deserializer {
            parser: Parser::new(input.chars()),
            many,
        }
    }

    pub fn from_str_many(input: &'de str) -> Self {
        Deserializer::new(input, true)
    }

    pub fn from_str(input: &'de str) -> Self {
        Deserializer::new(input, false)
    }
}

pub fn from_str_many<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str_many(s);

    T::deserialize(&mut deserializer)
}

pub fn from_str<'a, T>(s: &'a str) -> Result<T, Error>
where
    T: Deserialize<'a>,
{
    let mut deserializer = Deserializer::from_str(s);

    let (ev, _mark) = deserializer.parser.next()?;
    assert_eq!(ev, Event::StreamStart);

    let (ev, _mark) = deserializer.parser.peek()?;

    if *ev == Event::DocumentStart {
        deserializer.parser.next()?;
    }

    let res = T::deserialize(&mut deserializer);

    let (ev, _mark) = deserializer.parser.peek()?;

    if *ev == Event::DocumentEnd {
        deserializer.parser.next()?;
    }

    let (ev, _mark) = deserializer.parser.next()?;
    assert_eq!(ev, Event::StreamEnd);

    res
}

impl<'de> serde::de::Deserializer<'de> for &mut Deserializer<'de> {
    type Error = Error;

    fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
        let (ev, mark) = self.parser.next()?;

        let value = match ev {
            Event::SequenceStart(_) => visitor
                .visit_seq(ArrayAccess::new(self))
                .map_err(|err: Error| err + mark)?,
            Event::StreamStart if self.many => visitor
                .visit_seq(ArrayAccess::new(self))
                .map_err(|err: Error| err + mark)?,
            _ => {
                if self.many {
                    return Err(Error::from_event(ev, mark, "the start of the stream"));
                } else {
                    return Err(Error::from_event(ev, mark, "the start of a sequence"));
                }
            }
        };

        if self.many {
            let (ev, mark) = self.parser.next()?;

            if ev != Event::StreamEnd {
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
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        unimplemented!()
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
}

impl<'a, 'de> ArrayAccess<'a, 'de> {
    fn new(de: &'a mut Deserializer<'de>) -> Self {
        Self { de }
    }
}

impl<'de, 'a> SeqAccess<'de> for ArrayAccess<'a, 'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        let (ev, _mark) = self.de.parser.peek()?;

        if self.de.many {
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
}
