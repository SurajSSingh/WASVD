use serde::{Deserialize, Serialize};
use specta::Type;
use wast::token::{Float32, Float64};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize, Deserialize, Type,
)]
pub struct SerializedNumber {
    lower_bits: u32,
    upper_bits: u32,
}

impl From<u32> for SerializedNumber {
    fn from(value: u32) -> Self {
        Self {
            lower_bits: u32::from_ne_bytes(value.to_ne_bytes()),
            upper_bits: 0,
        }
    }
}

impl From<i32> for SerializedNumber {
    fn from(value: i32) -> Self {
        Self {
            lower_bits: u32::from_ne_bytes(value.to_ne_bytes()),
            upper_bits: 0,
        }
    }
}

impl From<u64> for SerializedNumber {
    fn from(value: u64) -> Self {
        let bytes = value.to_ne_bytes();
        let (lower, upper) = (
            [bytes[0], bytes[1], bytes[2], bytes[3]],
            [bytes[4], bytes[5], bytes[6], bytes[7]],
        );
        Self {
            lower_bits: u32::from_ne_bytes(lower),
            upper_bits: u32::from_ne_bytes(upper),
        }
    }
}

impl From<i64> for SerializedNumber {
    fn from(value: i64) -> Self {
        let bytes = value.to_ne_bytes();
        let (lower, upper) = (
            [bytes[0], bytes[1], bytes[2], bytes[3]],
            [bytes[4], bytes[5], bytes[6], bytes[7]],
        );
        Self {
            lower_bits: u32::from_ne_bytes(lower),
            upper_bits: u32::from_ne_bytes(upper),
        }
    }
}

impl From<Float32> for SerializedNumber {
    fn from(value: Float32) -> Self {
        Self {
            lower_bits: u32::from_ne_bytes(value.bits.to_ne_bytes()),
            upper_bits: 0,
        }
    }
}

impl From<Float64> for SerializedNumber {
    fn from(value: Float64) -> Self {
        let bytes = value.bits.to_ne_bytes();
        let (lower, upper) = (
            [bytes[0], bytes[1], bytes[2], bytes[3]],
            [bytes[4], bytes[5], bytes[6], bytes[7]],
        );
        Self {
            lower_bits: u32::from_ne_bytes(lower),
            upper_bits: u32::from_ne_bytes(upper),
        }
    }
}

impl<T> From<Option<T>> for SerializedNumber
where
    SerializedNumber: From<T>,
{
    fn from(value: Option<T>) -> Self {
        value.map(SerializedNumber::from).unwrap_or_default()
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
