use serde::{Deserialize, Serialize};
use specta::Type;
use wast::token::{Float32, Float64};

use crate::{error::WatError, marker::SerializableWatType};

macro_rules! four_byte_array {
    ($array:ident, $start:literal) => {
        [
            $array[$start],
            $array[$start + 1],
            $array[$start + 2],
            $array[$start + 3],
        ]
    };
}

/// A number serialized as an array of bytes in big-endian order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SerializedNumber {
    first_bytes: [u8; 4],
    second_bytes: Option<[u8; 4]>,
    typ: SerializableWatType,
}

impl From<i32> for SerializedNumber {
    fn from(value: i32) -> Self {
        Self {
            first_bytes: value.to_be_bytes(),
            second_bytes: None,
            typ: SerializableWatType::I32,
        }
    }
}

impl From<i64> for SerializedNumber {
    fn from(value: i64) -> Self {
        let bytes = value.to_be_bytes();
        Self {
            first_bytes: four_byte_array!(bytes, 0),
            second_bytes: Some(four_byte_array!(bytes, 4)),
            typ: SerializableWatType::I64,
        }
    }
}

impl From<Float32> for SerializedNumber {
    fn from(value: Float32) -> Self {
        Self {
            first_bytes: value.bits.to_be_bytes(),
            second_bytes: None,
            typ: SerializableWatType::F32,
        }
    }
}

impl From<Float64> for SerializedNumber {
    fn from(value: Float64) -> Self {
        let bytes = value.bits.to_ne_bytes();
        Self {
            first_bytes: four_byte_array!(bytes, 0),
            second_bytes: Some(four_byte_array!(bytes, 4)),
            typ: SerializableWatType::F64,
        }
    }
}

impl<T> From<Option<T>> for SerializedNumber
where
    SerializedNumber: From<T>,
{
    fn from(value: Option<T>) -> Self {
        value.map(SerializedNumber::from).unwrap_or(0.into())
    }
}

impl<T> From<&T> for SerializedNumber
where
    T: Copy,
    SerializedNumber: From<T>,
{
    fn from(value: &T) -> Self {
        SerializedNumber::from(*value)
    }
}

impl TryFrom<SerializedNumber> for u32 {
    type Error = WatError;

    fn try_from(value: SerializedNumber) -> Result<Self, Self::Error> {
        if value
            .second_bytes
            .is_some_and(|bytes| bytes.iter().any(|x| x != &0))
        {
            Err(WatError::number_to_large(&value))
        } else {
            Ok(u32::from_be_bytes(value.first_bytes))
        }
    }
}
