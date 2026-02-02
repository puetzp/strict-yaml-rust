use crate::{
    emitter::{escape_str, need_quotes, StrictYamlEmitter},
    serde::error::Error,
    strict_yaml::{self, StrictYaml},
};
use serde::ser::{
    self, Serialize, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant,
};
use std::fmt;

pub fn to_strict_yaml<T: Serialize>(value: T) -> Result<StrictYaml, Error> {
    value.serialize(strict_yaml::serde::ser::Serializer)
}

pub fn to_string<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut out = String::new();

    let mut serializer = Serializer {
        emitter: StrictYamlEmitter::new(&mut out),
        scope: None,
        multi_doc: false,
    };

    write!(serializer.emitter.writer, "---")?;
    writeln!(serializer.emitter.writer)?;

    value.serialize(&mut serializer)?;

    Ok(out)
}

pub fn to_string_many<T>(value: &T) -> Result<String, Error>
where
    T: Serialize,
{
    let mut out = String::new();

    let mut serializer = Serializer {
        emitter: StrictYamlEmitter::new(&mut out),
        scope: None,
        multi_doc: true,
    };

    value.serialize(&mut serializer)?;

    Ok(out)
}

#[derive(Debug, PartialEq)]
enum Scope {
    Root,
    Key,
    Map,
    Seq,
}

pub struct Serializer<'a> {
    emitter: StrictYamlEmitter<'a>,
    scope: Option<Scope>,
    multi_doc: bool,
}

fn write_str<T: fmt::Display>(
    serializer: &mut Serializer,
    v: T,
    maybe_multi_line: bool,
) -> Result<(), Error> {
    let s = v.to_string();

    if serializer.scope == Some(Scope::Key) {
        serializer.emitter.writer.write_char(' ')?;
    }

    if maybe_multi_line && s.ends_with('\n') {
        serializer.emitter.writer.write_char('|')?;
        writeln!(serializer.emitter.writer)?;

        let level_delta = if serializer.emitter.level < 0 { 2 } else { 1 };

        serializer.emitter.level += level_delta;

        for line in s.lines() {
            serializer.emitter.write_indent()?;
            serializer.emitter.writer.write_str(line)?;
            writeln!(serializer.emitter.writer)?;
        }

        serializer.emitter.level -= level_delta;
    } else {
        if need_quotes(&s) {
            escape_str(serializer.emitter.writer, &s)?;
        } else {
            serializer.emitter.writer.write_str(&s)?;
        }

        if serializer.scope != Some(Scope::Map) {
            writeln!(serializer.emitter.writer)?;
        }
    }

    Ok(())
}

impl ser::Serializer for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, false)
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        write_str(&mut *self, v, true)
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        let s = std::str::from_utf8(v)?;

        write_str(&mut *self, s, true)
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        variant.serialize(self)
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        self.emitter.level += 1;

        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        variant.serialize(&mut *self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        old_scope = self.scope.replace(Scope::Key);
        let v = value.serialize(&mut *self)?;
        self.scope = old_scope.take();

        self.emitter.level -= 1;

        Ok(v)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        if self.multi_doc && self.scope.is_none() {
            self.scope = Some(Scope::Root);
        }

        if self.scope != Some(Scope::Root) {
            if len == Some(0) {
                write!(self.emitter.writer, "[]")?;
            } else {
                self.emitter.level += 1;
            }
        }

        if self.scope == Some(Scope::Map) || self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.emitter.level += 1;

        let mut old_scope = self.scope.replace(Scope::Map);
        variant.serialize(&mut *self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        self.scope.replace(Scope::Key);

        self.serialize_seq(Some(len))
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        if len == Some(0) {
            write!(self.emitter.writer, "{{}}")?;
        } else {
            self.emitter.level += 1;
        }

        if self.scope == Some(Scope::Key) {
            writeln!(self.emitter.writer)?;
        }

        Ok(self)
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        self.emitter.level += 1;

        let mut old_scope = self.scope.replace(Scope::Map);
        variant.serialize(&mut *self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        self.scope.replace(Scope::Key);

        self.serialize_map(Some(len))
    }
}

impl SerializeSeq for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope == Some(Scope::Root) {
            write!(self.emitter.writer, "---")?;
            writeln!(self.emitter.writer)?;
            value.serialize(&mut **self)?;
        } else {
            if self.scope != Some(Scope::Seq) {
                self.emitter.write_indent()?;
            } else {
                self.scope = None;
            }

            write!(self.emitter.writer, "- ")?;

            let mut old_scope = self.scope.replace(Scope::Seq);
            value.serialize(&mut **self)?;
            self.scope = old_scope.take();
        }

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;

        if self.scope == Some(Scope::Root) {
            self.scope = None;
        }

        Ok(())
    }
}

impl SerializeTuple for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        write!(self.emitter.writer, "- ")?;

        let mut old_scope = self.scope.replace(Scope::Seq);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeTupleStruct for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        write!(self.emitter.writer, "- ")?;

        let mut old_scope = self.scope.replace(Scope::Seq);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeTupleVariant for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        write!(self.emitter.writer, "- ")?;

        let mut old_scope = self.scope.replace(Scope::Seq);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeMap for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        key.serialize(&mut **self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        let mut old_scope = self.scope.replace(Scope::Key);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeStruct for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        key.serialize(&mut **self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        old_scope = self.scope.replace(Scope::Key);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}

impl SerializeStructVariant for &'_ mut Serializer<'_> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + ser::Serialize,
    {
        if self.scope != Some(Scope::Seq) {
            self.emitter.write_indent()?;
        } else {
            self.scope = None;
        }

        let mut old_scope = self.scope.replace(Scope::Map);
        key.serialize(&mut **self)?;
        self.scope = old_scope.take();

        write!(self.emitter.writer, ":")?;

        old_scope = self.scope.replace(Scope::Key);
        value.serialize(&mut **self)?;
        self.scope = old_scope.take();

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        self.emitter.level -= 1;
        Ok(())
    }
}
