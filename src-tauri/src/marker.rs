//! This module holds all marker enums (i.e., all enums that behave like tags or C-style enums).
//! The main requirement is to have the Copy-trait.
//!
//! The reason to use marker enums is to make it easier to check on the TypeScript side.

use serde::{Deserialize, Serialize};
use specta::Type;
use wast::core::Instruction;

use crate::error::{self, WatError, WatResult};

/// All Wat types that can be (currently) serialized.
///
/// ## Limitations
/// All except [ValType::Ref] are supported, but must explicity convert.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, derive_more::Display)]
pub enum SerializableWatType {
    I32,
    I64,
    F32,
    F64,
    V128,
}

impl SerializableWatType {
    pub fn try_type_match(&self, other: &SerializableWatType) -> WatResult<()> {
        if self == other {
            Ok(())
        } else {
            Err(WatError::type_error(self, other))
        }
    }
}

impl<'a> TryFrom<wast::core::ValType<'a>> for SerializableWatType {
    type Error = error::WatError;

    /// Try to go from [ValType] to [SerializableWatType]
    fn try_from(value: wast::core::ValType) -> Result<Self, Self::Error> {
        use wast::core::ValType;
        match value {
            ValType::I32 => Ok(SerializableWatType::I32),
            ValType::I64 => Ok(SerializableWatType::I64),
            ValType::F32 => Ok(SerializableWatType::F32),
            ValType::F64 => Ok(SerializableWatType::F64),
            ValType::V128 => Ok(SerializableWatType::V128),
            ValType::Ref(_) => Err(error::WatError::unimplemented_error("Cannot use Ref type")),
        }
    }
}

/// The kind of number
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum NumberKind {
    /// Int that can be either signed or unsigned
    Bytes,
    /// Int that is signed
    UnsignedInt,
    /// Int that is unsigned
    SignedInt,
    /// Floating point number
    Float,
}

/// The kind of byte
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ByteKind {
    Bits8 = 0,
    Bits16 = 1,
    Bits32 = 2,
    Bits64 = 3,
}

impl ByteKind {
    /// Produce [ByteKind] from an alignment number
    pub fn from_alignment(value: u32) -> Self {
        match value {
            1 => ByteKind::Bits8,
            2 => ByteKind::Bits16,
            4 => ByteKind::Bits32,
            _ => ByteKind::Bits64,
        }
    }

    /// Produce [ByteKind] from a number of bits (8 bits per byte)
    pub fn from_bit_count(value: u32) -> Self {
        match value {
            8 => ByteKind::Bits8,
            16 => ByteKind::Bits16,
            32 => ByteKind::Bits32,
            _ => ByteKind::Bits64,
        }
    }

    /// Produce [ByteKind] from a number of byte
    pub fn from_byte_count(value: u32) -> Self {
        match value {
            1 => ByteKind::Bits8,
            2 => ByteKind::Bits16,
            4 => ByteKind::Bits32,
            _ => ByteKind::Bits64,
        }
    }
}

pub fn try_byte_count_from(instruction: &Instruction) -> Option<ByteKind> {
    match instruction {
        Instruction::I32Load8s(_)
        | Instruction::I32Store8(_)
        | Instruction::I64Store8(_)
        | Instruction::I32Load8u(_)
        | Instruction::I64Load8s(_)
        | Instruction::I64Load8u(_) => Some(ByteKind::Bits8),
        Instruction::I32Store16(_)
        | Instruction::I64Store16(_)
        | Instruction::I32Load16s(_)
        | Instruction::I32Load16u(_)
        | Instruction::I64Load16s(_)
        | Instruction::I64Load16u(_) => Some(ByteKind::Bits16),
        Instruction::I32Load(_)
        | Instruction::F32Load(_)
        | Instruction::I64Load32s(_)
        | Instruction::I64Load32u(_)
        | Instruction::I32Store(_)
        | Instruction::F32Store(_)
        | Instruction::I64Store32(_) => Some(ByteKind::Bits32),
        Instruction::I64Load(_)
        | Instruction::F64Load(_)
        | Instruction::I64Store(_)
        | Instruction::F64Store(_) => Some(ByteKind::Bits64),
        _ => None,
    }
}
/// Comparison operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ComparisonOperation {
    /// <0> == 0
    EqualZero,
    /// <0> == <1>
    Equal,
    /// <0> != <1>
    NotEqual,
    /// <0>_s < <1>_s
    LessThenSigned,
    /// <0>_u < <1>_u
    LessThenUnsigned,
    /// <0>_s > <1>_s
    GreaterThenSigned,
    /// <0>_u > <1>_u
    GreaterThenUnsigned,
    /// <0>_s <= <1>_s
    LessThenOrEqualToSigned,
    /// <0>_u <= <1>_u
    LessThenOrEqualToUnsigned,
    /// <0>_s >= <1>_s
    GreaterThenOrEqualToSigned,
    /// <0>_u >= <1>_u
    GreaterThenOrEqualToUnsigned,
}

pub fn try_comparison_from(instruction: &Instruction) -> Option<ComparisonOperation> {
    match instruction {
        Instruction::I32Eq | Instruction::I64Eq | Instruction::F32Eq | Instruction::F64Eq => {
            Some(ComparisonOperation::Equal)
        }
        Instruction::I32Eqz | Instruction::I64Eqz => Some(ComparisonOperation::EqualZero),
        Instruction::I32Ne | Instruction::I64Ne | Instruction::F32Ne | Instruction::F64Ne => {
            Some(ComparisonOperation::NotEqual)
        }
        Instruction::I32LtS | Instruction::I64LtS | Instruction::F32Lt | Instruction::F64Lt => {
            Some(ComparisonOperation::LessThenSigned)
        }
        Instruction::I32LtU | Instruction::I64LtU => Some(ComparisonOperation::LessThenUnsigned),
        Instruction::I32GtS | Instruction::I64GtS | Instruction::F32Gt | Instruction::F64Gt => {
            Some(ComparisonOperation::GreaterThenSigned)
        }
        Instruction::I32GtU | Instruction::I64GtU => Some(ComparisonOperation::GreaterThenUnsigned),
        Instruction::I32LeS | Instruction::I64LeS | Instruction::F32Le | Instruction::F64Le => {
            Some(ComparisonOperation::LessThenOrEqualToSigned)
        }
        Instruction::I32LeU | Instruction::I64LeU => {
            Some(ComparisonOperation::LessThenOrEqualToUnsigned)
        }
        Instruction::I32GeS | Instruction::I64GeS | Instruction::F32Ge | Instruction::F64Ge => {
            Some(ComparisonOperation::GreaterThenOrEqualToSigned)
        }
        Instruction::I32GeU | Instruction::I64GeU => {
            Some(ComparisonOperation::GreaterThenOrEqualToUnsigned)
        }
        _ => None,
    }
}

/// Arithmetic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum ArithmeticOperation {
    /// <0> + <1>
    Addition,
    /// <0> - <1>
    Subtraction,
    /// <0> * <1>
    Multiplication,
    /// <0>_s / <1>_s
    DivisonSigned,
    /// <0>_u / <1>_u
    DivisonUnsigned,
    /// <0>_s % <1>_s
    RemainderSigned,
    /// <0>_u % <1>_u
    RemainderUnsigned,
}

pub fn try_arithmetic_from(instruction: &Instruction) -> Option<ArithmeticOperation> {
    match instruction {
        Instruction::I32Add | Instruction::I64Add | Instruction::F32Add | Instruction::F64Add => {
            Some(ArithmeticOperation::Addition)
        }
        Instruction::I32Sub | Instruction::I64Sub | Instruction::F32Sub | Instruction::F64Sub => {
            Some(ArithmeticOperation::Subtraction)
        }
        Instruction::I32Mul | Instruction::I64Mul | Instruction::F32Mul | Instruction::F64Mul => {
            Some(ArithmeticOperation::Multiplication)
        }
        Instruction::I32DivS | Instruction::I64DivS | Instruction::F32Div | Instruction::F64Div => {
            Some(ArithmeticOperation::DivisonSigned)
        }
        Instruction::I32DivU | Instruction::I64DivU => Some(ArithmeticOperation::DivisonUnsigned),
        Instruction::I32RemS | Instruction::I64RemS => Some(ArithmeticOperation::RemainderSigned),
        Instruction::I32RemU | Instruction::I64RemU => Some(ArithmeticOperation::RemainderUnsigned),
        _ => None,
    }
}

/// Bitwise operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum BitwiseOperation {
    CountLeadingZero,
    CountTrailingZero,
    CountNonZero,
    /// <0> & <1>
    And,
    /// <0> | <1>
    Or,
    /// <0> ^ <1>
    Xor,
    /// <0> << <1>
    ShiftLeft,
    /// <0>_s >> <1>
    ShiftRightSigned,
    /// <0>_u >> <1>
    ShiftRightUnsigned,
    /// <0>_u rotate left by <1>_u
    RotateLeft,
    /// <0>_u rotate right by <1>_u
    RotateRight,
}

pub fn try_bitwise_from(instruction: &Instruction) -> Option<BitwiseOperation> {
    match instruction {
        Instruction::I32Clz | Instruction::I64Clz => Some(BitwiseOperation::CountLeadingZero),
        Instruction::I32Ctz | Instruction::I64Ctz => Some(BitwiseOperation::CountTrailingZero),
        Instruction::I32Popcnt | Instruction::I64Popcnt => Some(BitwiseOperation::CountNonZero),
        Instruction::I32And | Instruction::I64And => Some(BitwiseOperation::And),
        Instruction::I32Or | Instruction::I64Or => Some(BitwiseOperation::Or),
        Instruction::I32Xor | Instruction::I64Xor => Some(BitwiseOperation::Xor),
        Instruction::I32Shl | Instruction::I64Shl => Some(BitwiseOperation::ShiftLeft),
        Instruction::I32ShrS | Instruction::I64ShrS => Some(BitwiseOperation::ShiftRightSigned),
        Instruction::I32ShrU | Instruction::I64ShrU => Some(BitwiseOperation::ShiftRightUnsigned),
        Instruction::I32Rotl | Instruction::I64Rotl => Some(BitwiseOperation::RotateLeft),
        Instruction::I32Rotr | Instruction::I64Rotr => Some(BitwiseOperation::RotateRight),
        _ => None,
    }
}

/// Bitwise operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum FloatOperation {
    /// |<0>|
    AbsoluteValue,
    /// -(<0>)
    Negation,
    /// round_up_to_int(<0>)
    Ceiling,
    /// round_down_to_int(<0>)
    Floor,
    /// round_nearest_int_to_zero(<0>)
    Truncate,
    /// round_nearest_int_to_even(<0>)
    Nearest,
    /// √<0>
    SquareRoot,
    /// min(<0>, <1>)
    Minimum,
    /// max(<0>, <1>)
    Maximum,
    /// sign(<0>) == sign(<1>) ? <0> else -(<0>)
    CopySign,
}

pub fn try_float_op_from(instruction: &Instruction) -> Option<FloatOperation> {
    match instruction {
        Instruction::F32Abs | Instruction::F64Abs => Some(FloatOperation::AbsoluteValue),
        Instruction::F32Neg | Instruction::F64Neg => Some(FloatOperation::Negation),
        Instruction::F32Ceil | Instruction::F64Ceil => Some(FloatOperation::Ceiling),
        Instruction::F32Floor | Instruction::F64Floor => Some(FloatOperation::Floor),
        Instruction::F32Trunc | Instruction::F64Trunc => Some(FloatOperation::Truncate),
        Instruction::F32Nearest | Instruction::F64Nearest => Some(FloatOperation::Nearest),
        Instruction::F32Sqrt | Instruction::F64Sqrt => Some(FloatOperation::SquareRoot),
        Instruction::F32Min | Instruction::F64Min => Some(FloatOperation::Minimum),
        Instruction::F32Max | Instruction::F64Max => Some(FloatOperation::Maximum),
        Instruction::F32Copysign | Instruction::F64Copysign => Some(FloatOperation::CopySign),
        _ => None,
    }
}

/// Numeric Conversion Type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
// TODO: Make macro to make it easier to write
pub enum NumericConversionKind {
    WrapInt,
    SignedTruncF32ToI32,
    UnsignedTruncF32ToI32,
    SignedTruncF64ToI32,
    UnsignedTruncF64ToI32,
    SignedTruncF32ToI64,
    UnsignedTruncF32ToI64,
    SignedTruncF64ToI64,
    UnsignedTruncF64ToI64,
    SignedExtend,
    UnsignedExtend,
    SignedConvertI32ToF32,
    UnsignedConvertI32ToF32,
    SignedConvertI64ToF32,
    UnsignedConvertI64ToF32,
    SignedConvertI32ToF64,
    UnsignedConvertI32ToF64,
    SignedConvertI64ToF64,
    UnsignedConvertI64ToF64,
    DemoteFloat,
    PromoteFloat,
    Reinterpret32FToI,
    Reinterpret32IToF,
    Reinterpret64FToI,
    Reinterpret64IToF,
}

pub fn try_cast_kind_from(instruction: &Instruction) -> Option<NumericConversionKind> {
    match instruction {
        Instruction::I32WrapI64 => Some(NumericConversionKind::WrapInt),
        Instruction::I32TruncF32S => Some(NumericConversionKind::SignedTruncF32ToI32),
        Instruction::I32TruncF32U => Some(NumericConversionKind::UnsignedTruncF32ToI32),
        Instruction::I32TruncF64S => Some(NumericConversionKind::SignedTruncF64ToI32),
        Instruction::I32TruncF64U => Some(NumericConversionKind::UnsignedTruncF64ToI32),
        Instruction::I64ExtendI32S => Some(NumericConversionKind::SignedExtend),
        Instruction::I64ExtendI32U => Some(NumericConversionKind::UnsignedExtend),
        Instruction::I64TruncF32S => Some(NumericConversionKind::SignedTruncF32ToI64),
        Instruction::I64TruncF32U => Some(NumericConversionKind::UnsignedTruncF32ToI64),
        Instruction::I64TruncF64S => Some(NumericConversionKind::SignedTruncF64ToI64),
        Instruction::I64TruncF64U => Some(NumericConversionKind::UnsignedTruncF64ToI64),
        Instruction::F32ConvertI32S => Some(NumericConversionKind::SignedConvertI32ToF32),
        Instruction::F32ConvertI32U => Some(NumericConversionKind::UnsignedConvertI32ToF32),
        Instruction::F32ConvertI64S => Some(NumericConversionKind::SignedConvertI64ToF32),
        Instruction::F32ConvertI64U => Some(NumericConversionKind::UnsignedConvertI64ToF32),
        Instruction::F32DemoteF64 => Some(NumericConversionKind::DemoteFloat),
        Instruction::F64ConvertI32S => Some(NumericConversionKind::SignedConvertI32ToF64),
        Instruction::F64ConvertI32U => Some(NumericConversionKind::UnsignedConvertI32ToF64),
        Instruction::F64ConvertI64S => Some(NumericConversionKind::SignedConvertI64ToF64),
        Instruction::F64ConvertI64U => Some(NumericConversionKind::UnsignedConvertI64ToF64),
        Instruction::F64PromoteF32 => Some(NumericConversionKind::PromoteFloat),
        Instruction::I32ReinterpretF32 => Some(NumericConversionKind::Reinterpret32FToI),
        Instruction::I64ReinterpretF64 => Some(NumericConversionKind::Reinterpret64FToI),
        Instruction::F32ReinterpretI32 => Some(NumericConversionKind::Reinterpret32IToF),
        Instruction::F64ReinterpretI64 => Some(NumericConversionKind::Reinterpret64IToF),
        _ => None,
    }
}

/// Simple Instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum SimpleInstruction {
    Unreachable,
    Nop,
    Drop,
    Return,
}

pub fn try_simple_instruction_from(instruction: &Instruction) -> Option<SimpleInstruction> {
    match instruction {
        Instruction::Unreachable => Some(SimpleInstruction::Unreachable),
        Instruction::Nop => Some(SimpleInstruction::Nop),
        Instruction::Drop => Some(SimpleInstruction::Drop),
        Instruction::Return => Some(SimpleInstruction::Return),
        _ => None,
    }
}

/// Control flow instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum BlockKind {
    Block,
    If,
    Else,
    Loop,
    End,
}

pub fn try_block_kind_from(instruction: &Instruction) -> Option<BlockKind> {
    match instruction {
        Instruction::Block(_) => Some(BlockKind::Block),
        Instruction::If(_) => Some(BlockKind::If),
        Instruction::Else(_) => Some(BlockKind::Else),
        Instruction::Loop(_) => Some(BlockKind::Loop),
        Instruction::End(_) => Some(BlockKind::End),
        _ => None,
    }
}

/// Memory Instructions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum DataInstruction {
    GetLocal,
    GetGlobal,
    SetLocal,
    SetGlobal,
    TeeLocal,
    GetMemorySize,
    SetMemorySize,
}

pub fn try_data_instruction_from(instruction: &Instruction) -> Option<DataInstruction> {
    match instruction {
        Instruction::LocalGet(_) => Some(DataInstruction::GetLocal),
        Instruction::LocalSet(_) => Some(DataInstruction::SetLocal),
        Instruction::LocalTee(_) => Some(DataInstruction::TeeLocal),
        Instruction::GlobalGet(_) => Some(DataInstruction::SetGlobal),
        Instruction::GlobalSet(_) => Some(DataInstruction::GetGlobal),
        Instruction::MemorySize(_) => Some(DataInstruction::GetMemorySize),
        Instruction::MemoryGrow(_) => Some(DataInstruction::SetMemorySize),
        _ => None,
    }
}

/// Kind of numeric operation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, derive_more::From)]
pub enum NumericOperationKind {
    Comparison(ComparisonOperation),
    Arithmetic(ArithmeticOperation),
    Bitwise(BitwiseOperation),
    Float(FloatOperation),
}
