use crate::{
    emitter::{escape_str, need_quotes, StrictYamlEmitter},
    serde::error::Error,
};
use serde::{ser, Serialize};

pub struct Serializer {
    emitter: StrictYamlEmitter,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        emitter: StrictYamlEmitter::new(String::new()),
    };

    value.serialize(&mut serializer)?;

    Ok(serializer.emitter.writer)
}

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;
}
