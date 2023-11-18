//! This module holds the data types handling converting from Wat instructions to a unified instruction for the interpreter.

use crate::helper::SerializedNumber;
use crate::marker::{
    try_arithmetic_from, try_bitwise_from, try_block_kind_from, try_byte_count_from,
    try_cast_kind_from, try_comparison_from, try_data_instruction_from, try_float_op_from,
    try_simple_instruction_from, ArithmeticOperation, BitwiseOperation, BlockKind, ByteKind,
    ComparisonOperation, DataInstruction, FloatOperation, NumericConversionKind,
    SerializableWatType, SimpleInstruction,
};

use crate::error::{self, WatError};

use serde::{Deserialize, Serialize};
use specta::Type;
use wast::{
    self,
    core::{FunctionType, Instruction},
    token::Index,
};

pub fn data_type_of_instruction(instruction: &Instruction) -> Option<SerializableWatType> {
    match instruction {
        Instruction::F32Load(_)
        | Instruction::F32Store(_)
        | Instruction::F32Const(_)
        | Instruction::F32Abs
        | Instruction::F32Neg
        | Instruction::F32Ceil
        | Instruction::F32Floor
        | Instruction::F32Trunc
        | Instruction::F32Nearest
        | Instruction::F32Sqrt
        | Instruction::F32Add
        | Instruction::F32Sub
        | Instruction::F32Mul
        | Instruction::F32Div
        | Instruction::F32Min
        | Instruction::F32Max
        | Instruction::F32Copysign
        | Instruction::F32Eq
        | Instruction::F32Ne
        | Instruction::F32Lt
        | Instruction::F32Gt
        | Instruction::F32Le
        | Instruction::F32Ge
        | Instruction::F32ConvertI32S
        | Instruction::F32ConvertI32U
        | Instruction::F32ConvertI64S
        | Instruction::F32ConvertI64U
        | Instruction::F32DemoteF64
        | Instruction::F32ReinterpretI32 => Some(SerializableWatType::F32),
        Instruction::F64Load(_)
        | Instruction::F64Store(_)
        | Instruction::F64Const(_)
        | Instruction::F64Abs
        | Instruction::F64Neg
        | Instruction::F64Ceil
        | Instruction::F64Floor
        | Instruction::F64Trunc
        | Instruction::F64Nearest
        | Instruction::F64Sqrt
        | Instruction::F64Add
        | Instruction::F64Sub
        | Instruction::F64Mul
        | Instruction::F64Div
        | Instruction::F64Min
        | Instruction::F64Max
        | Instruction::F64Copysign
        | Instruction::F64Eq
        | Instruction::F64Ne
        | Instruction::F64Lt
        | Instruction::F64Gt
        | Instruction::F64Le
        | Instruction::F64Ge
        | Instruction::F64ConvertI32S
        | Instruction::F64ConvertI32U
        | Instruction::F64PromoteF32
        | Instruction::F64ConvertI64S
        | Instruction::F64ConvertI64U
        | Instruction::F64ReinterpretI64 => Some(SerializableWatType::F64),
        Instruction::I32Load(_)
        | Instruction::I32Load8s(_)
        | Instruction::I32Load8u(_)
        | Instruction::I32Load16s(_)
        | Instruction::I32Load16u(_)
        | Instruction::I32Store(_)
        | Instruction::I32Store8(_)
        | Instruction::I32Store16(_)
        | Instruction::I32Const(_)
        | Instruction::I32Clz
        | Instruction::I32Ctz
        | Instruction::I32Popcnt
        | Instruction::I32Add
        | Instruction::I32Sub
        | Instruction::I32Mul
        | Instruction::I32DivS
        | Instruction::I32DivU
        | Instruction::I32RemS
        | Instruction::I32RemU
        | Instruction::I32And
        | Instruction::I32Or
        | Instruction::I32Xor
        | Instruction::I32Shl
        | Instruction::I32ShrS
        | Instruction::I32ShrU
        | Instruction::I32Rotl
        | Instruction::I32Rotr
        | Instruction::I32Eqz
        | Instruction::I32Eq
        | Instruction::I32Ne
        | Instruction::I32LtS
        | Instruction::I32LtU
        | Instruction::I32GtS
        | Instruction::I32GtU
        | Instruction::I32LeS
        | Instruction::I32LeU
        | Instruction::I32GeS
        | Instruction::I32GeU
        | Instruction::I32TruncF32S
        | Instruction::I32TruncF32U
        | Instruction::I32TruncF64S
        | Instruction::I32TruncF64U
        | Instruction::I32ReinterpretF32
        | Instruction::I32TruncSatF32S
        | Instruction::I32TruncSatF32U
        | Instruction::I32TruncSatF64S
        | Instruction::I32TruncSatF64U
        | Instruction::I32Extend8S
        | Instruction::I32Extend16S => Some(SerializableWatType::I32),
        Instruction::I64Load(_)
        | Instruction::I64Load8s(_)
        | Instruction::I64Load8u(_)
        | Instruction::I64Load16s(_)
        | Instruction::I64Load16u(_)
        | Instruction::I64Load32s(_)
        | Instruction::I64Load32u(_)
        | Instruction::I64Store(_)
        | Instruction::I64Store8(_)
        | Instruction::I64Store16(_)
        | Instruction::I64Store32(_)
        | Instruction::I64Const(_)
        | Instruction::I64Clz
        | Instruction::I64Ctz
        | Instruction::I64Popcnt
        | Instruction::I64Add
        | Instruction::I64Sub
        | Instruction::I64Mul
        | Instruction::I64DivS
        | Instruction::I64DivU
        | Instruction::I64RemS
        | Instruction::I64RemU
        | Instruction::I64And
        | Instruction::I64Or
        | Instruction::I64Xor
        | Instruction::I64Shl
        | Instruction::I64ShrS
        | Instruction::I64ShrU
        | Instruction::I64Rotl
        | Instruction::I64Rotr
        | Instruction::I64Eqz
        | Instruction::I64Eq
        | Instruction::I64Ne
        | Instruction::I64LtS
        | Instruction::I64LtU
        | Instruction::I64GtS
        | Instruction::I64GtU
        | Instruction::I64LeS
        | Instruction::I64LeU
        | Instruction::I64GeS
        | Instruction::I64GeU
        | Instruction::I32WrapI64
        | Instruction::I64ExtendI32S
        | Instruction::I64ExtendI32U
        | Instruction::I64TruncF32S
        | Instruction::I64TruncF32U
        | Instruction::I64TruncF64S
        | Instruction::I64TruncF64U
        | Instruction::I64ReinterpretF64
        | Instruction::I64TruncSatF32S
        | Instruction::I64TruncSatF32U
        | Instruction::I64TruncSatF64S
        | Instruction::I64TruncSatF64U
        | Instruction::I64Extend8S
        | Instruction::I64Extend16S
        | Instruction::I64Extend32S => Some(SerializableWatType::I64),
        // Instruction::MemoryAtomicNotify(_) => todo!(),
        // Instruction::MemoryAtomicWait32(_) => todo!(),
        // Instruction::MemoryAtomicWait64(_) => todo!(),
        // Instruction::AtomicFence => todo!(),
        // Instruction::I32AtomicLoad(_) => todo!(),
        // Instruction::I64AtomicLoad(_) => todo!(),
        // Instruction::I32AtomicLoad8u(_) => todo!(),
        // Instruction::I32AtomicLoad16u(_) => todo!(),
        // Instruction::I64AtomicLoad8u(_) => todo!(),
        // Instruction::I64AtomicLoad16u(_) => todo!(),
        // Instruction::I64AtomicLoad32u(_) => todo!(),
        // Instruction::I32AtomicStore(_) => todo!(),
        // Instruction::I64AtomicStore(_) => todo!(),
        // Instruction::I32AtomicStore8(_) => todo!(),
        // Instruction::I32AtomicStore16(_) => todo!(),
        // Instruction::I64AtomicStore8(_) => todo!(),
        // Instruction::I64AtomicStore16(_) => todo!(),
        // Instruction::I64AtomicStore32(_) => todo!(),
        // Instruction::I32AtomicRmwAdd(_) => todo!(),
        // Instruction::I64AtomicRmwAdd(_) => todo!(),
        // Instruction::I32AtomicRmw8AddU(_) => todo!(),
        // Instruction::I32AtomicRmw16AddU(_) => todo!(),
        // Instruction::I64AtomicRmw8AddU(_) => todo!(),
        // Instruction::I64AtomicRmw16AddU(_) => todo!(),
        // Instruction::I64AtomicRmw32AddU(_) => todo!(),
        // Instruction::I32AtomicRmwSub(_) => todo!(),
        // Instruction::I64AtomicRmwSub(_) => todo!(),
        // Instruction::I32AtomicRmw8SubU(_) => todo!(),
        // Instruction::I32AtomicRmw16SubU(_) => todo!(),
        // Instruction::I64AtomicRmw8SubU(_) => todo!(),
        // Instruction::I64AtomicRmw16SubU(_) => todo!(),
        // Instruction::I64AtomicRmw32SubU(_) => todo!(),
        // Instruction::I32AtomicRmwAnd(_) => todo!(),
        // Instruction::I64AtomicRmwAnd(_) => todo!(),
        // Instruction::I32AtomicRmw8AndU(_) => todo!(),
        // Instruction::I32AtomicRmw16AndU(_) => todo!(),
        // Instruction::I64AtomicRmw8AndU(_) => todo!(),
        // Instruction::I64AtomicRmw16AndU(_) => todo!(),
        // Instruction::I64AtomicRmw32AndU(_) => todo!(),
        // Instruction::I32AtomicRmwOr(_) => todo!(),
        // Instruction::I64AtomicRmwOr(_) => todo!(),
        // Instruction::I32AtomicRmw8OrU(_) => todo!(),
        // Instruction::I32AtomicRmw16OrU(_) => todo!(),
        // Instruction::I64AtomicRmw8OrU(_) => todo!(),
        // Instruction::I64AtomicRmw16OrU(_) => todo!(),
        // Instruction::I64AtomicRmw32OrU(_) => todo!(),
        // Instruction::I32AtomicRmwXor(_) => todo!(),
        // Instruction::I64AtomicRmwXor(_) => todo!(),
        // Instruction::I32AtomicRmw8XorU(_) => todo!(),
        // Instruction::I32AtomicRmw16XorU(_) => todo!(),
        // Instruction::I64AtomicRmw8XorU(_) => todo!(),
        // Instruction::I64AtomicRmw16XorU(_) => todo!(),
        // Instruction::I64AtomicRmw32XorU(_) => todo!(),
        // Instruction::I32AtomicRmwXchg(_) => todo!(),
        // Instruction::I64AtomicRmwXchg(_) => todo!(),
        // Instruction::I32AtomicRmw8XchgU(_) => todo!(),
        // Instruction::I32AtomicRmw16XchgU(_) => todo!(),
        // Instruction::I64AtomicRmw8XchgU(_) => todo!(),
        // Instruction::I64AtomicRmw16XchgU(_) => todo!(),
        // Instruction::I64AtomicRmw32XchgU(_) => todo!(),
        // Instruction::I32AtomicRmwCmpxchg(_) => todo!(),
        // Instruction::I64AtomicRmwCmpxchg(_) => todo!(),
        // Instruction::I32AtomicRmw8CmpxchgU(_) => todo!(),
        // Instruction::I32AtomicRmw16CmpxchgU(_) => todo!(),
        // Instruction::I64AtomicRmw8CmpxchgU(_) => todo!(),
        // Instruction::I64AtomicRmw16CmpxchgU(_) => todo!(),
        // Instruction::I64AtomicRmw32CmpxchgU(_) => todo!(),
        // Instruction::V128Load(_) => todo!(),
        // Instruction::V128Load8x8S(_) => todo!(),
        // Instruction::V128Load8x8U(_) => todo!(),
        // Instruction::V128Load16x4S(_) => todo!(),
        // Instruction::V128Load16x4U(_) => todo!(),
        // Instruction::V128Load32x2S(_) => todo!(),
        // Instruction::V128Load32x2U(_) => todo!(),
        // Instruction::V128Load8Splat(_) => todo!(),
        // Instruction::V128Load16Splat(_) => todo!(),
        // Instruction::V128Load32Splat(_) => todo!(),
        // Instruction::V128Load64Splat(_) => todo!(),
        // Instruction::V128Load32Zero(_) => todo!(),
        // Instruction::V128Load64Zero(_) => todo!(),
        // Instruction::V128Store(_) => todo!(),
        // Instruction::V128Load8Lane(_) => todo!(),
        // Instruction::V128Load16Lane(_) => todo!(),
        // Instruction::V128Load32Lane(_) => todo!(),
        // Instruction::V128Load64Lane(_) => todo!(),
        // Instruction::V128Store8Lane(_) => todo!(),
        // Instruction::V128Store16Lane(_) => todo!(),
        // Instruction::V128Store32Lane(_) => todo!(),
        // Instruction::V128Store64Lane(_) => todo!(),
        // Instruction::V128Const(_) => todo!(),
        // Instruction::I8x16Shuffle(_) => todo!(),
        // Instruction::I8x16ExtractLaneS(_) => todo!(),
        // Instruction::I8x16ExtractLaneU(_) => todo!(),
        // Instruction::I8x16ReplaceLane(_) => todo!(),
        // Instruction::I16x8ExtractLaneS(_) => todo!(),
        // Instruction::I16x8ExtractLaneU(_) => todo!(),
        // Instruction::I16x8ReplaceLane(_) => todo!(),
        // Instruction::I32x4ExtractLane(_) => todo!(),
        // Instruction::I32x4ReplaceLane(_) => todo!(),
        // Instruction::I64x2ExtractLane(_) => todo!(),
        // Instruction::I64x2ReplaceLane(_) => todo!(),
        // Instruction::F32x4ExtractLane(_) => todo!(),
        // Instruction::F32x4ReplaceLane(_) => todo!(),
        // Instruction::F64x2ExtractLane(_) => todo!(),
        // Instruction::F64x2ReplaceLane(_) => todo!(),
        // Instruction::I8x16Swizzle => todo!(),
        // Instruction::I8x16Splat => todo!(),
        // Instruction::I16x8Splat => todo!(),
        // Instruction::I32x4Splat => todo!(),
        // Instruction::I64x2Splat => todo!(),
        // Instruction::F32x4Splat => todo!(),
        // Instruction::F64x2Splat => todo!(),
        // Instruction::I8x16Eq => todo!(),
        // Instruction::I8x16Ne => todo!(),
        // Instruction::I8x16LtS => todo!(),
        // Instruction::I8x16LtU => todo!(),
        // Instruction::I8x16GtS => todo!(),
        // Instruction::I8x16GtU => todo!(),
        // Instruction::I8x16LeS => todo!(),
        // Instruction::I8x16LeU => todo!(),
        // Instruction::I8x16GeS => todo!(),
        // Instruction::I8x16GeU => todo!(),
        // Instruction::I16x8Eq => todo!(),
        // Instruction::I16x8Ne => todo!(),
        // Instruction::I16x8LtS => todo!(),
        // Instruction::I16x8LtU => todo!(),
        // Instruction::I16x8GtS => todo!(),
        // Instruction::I16x8GtU => todo!(),
        // Instruction::I16x8LeS => todo!(),
        // Instruction::I16x8LeU => todo!(),
        // Instruction::I16x8GeS => todo!(),
        // Instruction::I16x8GeU => todo!(),
        // Instruction::I32x4Eq => todo!(),
        // Instruction::I32x4Ne => todo!(),
        // Instruction::I32x4LtS => todo!(),
        // Instruction::I32x4LtU => todo!(),
        // Instruction::I32x4GtS => todo!(),
        // Instruction::I32x4GtU => todo!(),
        // Instruction::I32x4LeS => todo!(),
        // Instruction::I32x4LeU => todo!(),
        // Instruction::I32x4GeS => todo!(),
        // Instruction::I32x4GeU => todo!(),
        // Instruction::I64x2Eq => todo!(),
        // Instruction::I64x2Ne => todo!(),
        // Instruction::I64x2LtS => todo!(),
        // Instruction::I64x2GtS => todo!(),
        // Instruction::I64x2LeS => todo!(),
        // Instruction::I64x2GeS => todo!(),
        // Instruction::F32x4Eq => todo!(),
        // Instruction::F32x4Ne => todo!(),
        // Instruction::F32x4Lt => todo!(),
        // Instruction::F32x4Gt => todo!(),
        // Instruction::F32x4Le => todo!(),
        // Instruction::F32x4Ge => todo!(),
        // Instruction::F64x2Eq => todo!(),
        // Instruction::F64x2Ne => todo!(),
        // Instruction::F64x2Lt => todo!(),
        // Instruction::F64x2Gt => todo!(),
        // Instruction::F64x2Le => todo!(),
        // Instruction::F64x2Ge => todo!(),
        // Instruction::V128Not => todo!(),
        // Instruction::V128And => todo!(),
        // Instruction::V128Andnot => todo!(),
        // Instruction::V128Or => todo!(),
        // Instruction::V128Xor => todo!(),
        // Instruction::V128Bitselect => todo!(),
        // Instruction::V128AnyTrue => todo!(),
        // Instruction::I8x16Abs => todo!(),
        // Instruction::I8x16Neg => todo!(),
        // Instruction::I8x16Popcnt => todo!(),
        // Instruction::I8x16AllTrue => todo!(),
        // Instruction::I8x16Bitmask => todo!(),
        // Instruction::I8x16NarrowI16x8S => todo!(),
        // Instruction::I8x16NarrowI16x8U => todo!(),
        // Instruction::I8x16Shl => todo!(),
        // Instruction::I8x16ShrS => todo!(),
        // Instruction::I8x16ShrU => todo!(),
        // Instruction::I8x16Add => todo!(),
        // Instruction::I8x16AddSatS => todo!(),
        // Instruction::I8x16AddSatU => todo!(),
        // Instruction::I8x16Sub => todo!(),
        // Instruction::I8x16SubSatS => todo!(),
        // Instruction::I8x16SubSatU => todo!(),
        // Instruction::I8x16MinS => todo!(),
        // Instruction::I8x16MinU => todo!(),
        // Instruction::I8x16MaxS => todo!(),
        // Instruction::I8x16MaxU => todo!(),
        // Instruction::I8x16AvgrU => todo!(),
        // Instruction::I16x8ExtAddPairwiseI8x16S => todo!(),
        // Instruction::I16x8ExtAddPairwiseI8x16U => todo!(),
        // Instruction::I16x8Abs => todo!(),
        // Instruction::I16x8Neg => todo!(),
        // Instruction::I16x8Q15MulrSatS => todo!(),
        // Instruction::I16x8AllTrue => todo!(),
        // Instruction::I16x8Bitmask => todo!(),
        // Instruction::I16x8NarrowI32x4S => todo!(),
        // Instruction::I16x8NarrowI32x4U => todo!(),
        // Instruction::I16x8ExtendLowI8x16S => todo!(),
        // Instruction::I16x8ExtendHighI8x16S => todo!(),
        // Instruction::I16x8ExtendLowI8x16U => todo!(),
        // Instruction::I16x8ExtendHighI8x16u => todo!(),
        // Instruction::I16x8Shl => todo!(),
        // Instruction::I16x8ShrS => todo!(),
        // Instruction::I16x8ShrU => todo!(),
        // Instruction::I16x8Add => todo!(),
        // Instruction::I16x8AddSatS => todo!(),
        // Instruction::I16x8AddSatU => todo!(),
        // Instruction::I16x8Sub => todo!(),
        // Instruction::I16x8SubSatS => todo!(),
        // Instruction::I16x8SubSatU => todo!(),
        // Instruction::I16x8Mul => todo!(),
        // Instruction::I16x8MinS => todo!(),
        // Instruction::I16x8MinU => todo!(),
        // Instruction::I16x8MaxS => todo!(),
        // Instruction::I16x8MaxU => todo!(),
        // Instruction::I16x8AvgrU => todo!(),
        // Instruction::I16x8ExtMulLowI8x16S => todo!(),
        // Instruction::I16x8ExtMulHighI8x16S => todo!(),
        // Instruction::I16x8ExtMulLowI8x16U => todo!(),
        // Instruction::I16x8ExtMulHighI8x16U => todo!(),
        // Instruction::I32x4ExtAddPairwiseI16x8S => todo!(),
        // Instruction::I32x4ExtAddPairwiseI16x8U => todo!(),
        // Instruction::I32x4Abs => todo!(),
        // Instruction::I32x4Neg => todo!(),
        // Instruction::I32x4AllTrue => todo!(),
        // Instruction::I32x4Bitmask => todo!(),
        // Instruction::I32x4ExtendLowI16x8S => todo!(),
        // Instruction::I32x4ExtendHighI16x8S => todo!(),
        // Instruction::I32x4ExtendLowI16x8U => todo!(),
        // Instruction::I32x4ExtendHighI16x8U => todo!(),
        // Instruction::I32x4Shl => todo!(),
        // Instruction::I32x4ShrS => todo!(),
        // Instruction::I32x4ShrU => todo!(),
        // Instruction::I32x4Add => todo!(),
        // Instruction::I32x4Sub => todo!(),
        // Instruction::I32x4Mul => todo!(),
        // Instruction::I32x4MinS => todo!(),
        // Instruction::I32x4MinU => todo!(),
        // Instruction::I32x4MaxS => todo!(),
        // Instruction::I32x4MaxU => todo!(),
        // Instruction::I32x4DotI16x8S => todo!(),
        // Instruction::I32x4ExtMulLowI16x8S => todo!(),
        // Instruction::I32x4ExtMulHighI16x8S => todo!(),
        // Instruction::I32x4ExtMulLowI16x8U => todo!(),
        // Instruction::I32x4ExtMulHighI16x8U => todo!(),
        // Instruction::I64x2Abs => todo!(),
        // Instruction::I64x2Neg => todo!(),
        // Instruction::I64x2AllTrue => todo!(),
        // Instruction::I64x2Bitmask => todo!(),
        // Instruction::I64x2ExtendLowI32x4S => todo!(),
        // Instruction::I64x2ExtendHighI32x4S => todo!(),
        // Instruction::I64x2ExtendLowI32x4U => todo!(),
        // Instruction::I64x2ExtendHighI32x4U => todo!(),
        // Instruction::I64x2Shl => todo!(),
        // Instruction::I64x2ShrS => todo!(),
        // Instruction::I64x2ShrU => todo!(),
        // Instruction::I64x2Add => todo!(),
        // Instruction::I64x2Sub => todo!(),
        // Instruction::I64x2Mul => todo!(),
        // Instruction::I64x2ExtMulLowI32x4S => todo!(),
        // Instruction::I64x2ExtMulHighI32x4S => todo!(),
        // Instruction::I64x2ExtMulLowI32x4U => todo!(),
        // Instruction::I64x2ExtMulHighI32x4U => todo!(),
        // Instruction::F32x4Ceil => todo!(),
        // Instruction::F32x4Floor => todo!(),
        // Instruction::F32x4Trunc => todo!(),
        // Instruction::F32x4Nearest => todo!(),
        // Instruction::F32x4Abs => todo!(),
        // Instruction::F32x4Neg => todo!(),
        // Instruction::F32x4Sqrt => todo!(),
        // Instruction::F32x4Add => todo!(),
        // Instruction::F32x4Sub => todo!(),
        // Instruction::F32x4Mul => todo!(),
        // Instruction::F32x4Div => todo!(),
        // Instruction::F32x4Min => todo!(),
        // Instruction::F32x4Max => todo!(),
        // Instruction::F32x4PMin => todo!(),
        // Instruction::F32x4PMax => todo!(),
        // Instruction::F64x2Ceil => todo!(),
        // Instruction::F64x2Floor => todo!(),
        // Instruction::F64x2Trunc => todo!(),
        // Instruction::F64x2Nearest => todo!(),
        // Instruction::F64x2Abs => todo!(),
        // Instruction::F64x2Neg => todo!(),
        // Instruction::F64x2Sqrt => todo!(),
        // Instruction::F64x2Add => todo!(),
        // Instruction::F64x2Sub => todo!(),
        // Instruction::F64x2Mul => todo!(),
        // Instruction::F64x2Div => todo!(),
        // Instruction::F64x2Min => todo!(),
        // Instruction::F64x2Max => todo!(),
        // Instruction::F64x2PMin => todo!(),
        // Instruction::F64x2PMax => todo!(),
        // Instruction::I32x4TruncSatF32x4S => todo!(),
        // Instruction::I32x4TruncSatF32x4U => todo!(),
        // Instruction::F32x4ConvertI32x4S => todo!(),
        // Instruction::F32x4ConvertI32x4U => todo!(),
        // Instruction::I32x4TruncSatF64x2SZero => todo!(),
        // Instruction::I32x4TruncSatF64x2UZero => todo!(),
        // Instruction::F64x2ConvertLowI32x4S => todo!(),
        // Instruction::F64x2ConvertLowI32x4U => todo!(),
        // Instruction::F32x4DemoteF64x2Zero => todo!(),
        // Instruction::F64x2PromoteLowF32x4 => todo!(),
        // Instruction::I8x16RelaxedSwizzle => todo!(),
        // Instruction::I32x4RelaxedTruncF32x4S => todo!(),
        // Instruction::I32x4RelaxedTruncF32x4U => todo!(),
        // Instruction::I32x4RelaxedTruncF64x2SZero => todo!(),
        // Instruction::I32x4RelaxedTruncF64x2UZero => todo!(),
        // Instruction::F32x4RelaxedMadd => todo!(),
        // Instruction::F32x4RelaxedNmadd => todo!(),
        // Instruction::F64x2RelaxedMadd => todo!(),
        // Instruction::F64x2RelaxedNmadd => todo!(),
        // Instruction::I8x16RelaxedLaneselect => todo!(),
        // Instruction::I16x8RelaxedLaneselect => todo!(),
        // Instruction::I32x4RelaxedLaneselect => todo!(),
        // Instruction::I64x2RelaxedLaneselect => todo!(),
        // Instruction::F32x4RelaxedMin => todo!(),
        // Instruction::F32x4RelaxedMax => todo!(),
        // Instruction::F64x2RelaxedMin => todo!(),
        // Instruction::F64x2RelaxedMax => todo!(),
        // Instruction::I16x8RelaxedQ15mulrS => todo!(),
        // Instruction::I16x8RelaxedDotI8x16I7x16S => todo!(),
        // Instruction::I32x4RelaxedDotI8x16I7x16AddS => todo!(),
        _ =>
        /*TODO: Add others, all other types should either be V128 or Ref*/
        {
            None
        }
    }
}

pub fn is_64_bit_instruction(instruction: &Instruction) -> Option<bool> {
    match data_type_of_instruction(instruction) {
        Some(SerializableWatType::I32 | SerializableWatType::F32 | SerializableWatType::V128) => {
            Some(false)
        }
        Some(SerializableWatType::I64 | SerializableWatType::F64) => Some(true),
        None => None,
    }
}

/// Represents input and output of a block of instructions.
/// For functions, inputs are parameters and outputs are results.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
pub struct InputOutput {
    pub(crate) index: Option<String>,
    pub(crate) input: Vec<(Option<String>, SerializableWatType)>,
    pub(crate) output: Vec<SerializableWatType>,
}

impl InputOutput {
    pub fn set_name_if_none(&mut self, name: &str) -> bool {
        if self.index.is_none() {
            self.index = Some(name.to_string());
            return true;
        }
        false
    }
}

impl TryFrom<&wast::core::TypeUse<'_, FunctionType<'_>>> for InputOutput {
    type Error = error::WatError;
    fn try_from(value: &wast::core::TypeUse<'_, FunctionType<'_>>) -> Result<Self, Self::Error> {
        let index = value.index.map(|i| index_to_string(&i));
        Ok(if let Some(ft) = &value.inline {
            Self {
                index,
                input: ft
                    .params
                    .iter()
                    .map(|(id, _, vtype)| {
                        SerializableWatType::try_from(*vtype)
                            .map(|swt| (id.map(|i| i.name().to_string()), swt))
                    })
                    .collect::<Result<_, error::WatError>>()?,
                output: ft
                    .results
                    .iter()
                    .map(|vtype| SerializableWatType::try_from(*vtype))
                    .collect::<Result<Vec<_>, _>>()?,
            }
        } else {
            Self {
                index,
                input: Vec::default(),
                output: Vec::default(),
            }
        })
    }
}

/// Serialized instructions based on parts of [Instruction],
/// but is more generic over types (e.g. a single Add instruction that carries the type).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum SerializedInstruction {
    Simple(SimpleInstruction),
    Block {
        label: String,
        kind: BlockKind,
        inout: Option<InputOutput>,
    },
    Branch {
        default_label: String,
        other_labels: Vec<String>,
        is_conditional: bool,
    },
    Call {
        index: String,
        inout: InputOutput,
    },
    Data {
        kind: DataInstruction,
        location: String,
    },
    Memory {
        location: String,
        typ: SerializableWatType,
        count: ByteKind,
        offset: u32,
        alignment: ByteKind,
        is_storing: bool,
    },
    Const {
        typ: SerializableWatType,
        value: SerializedNumber,
    },
    Comparison {
        kind: ComparisonOperation,
        typ: SerializableWatType,
    },
    Arithmetic {
        kind: ArithmeticOperation,
        typ: SerializableWatType,
    },
    Bitwise {
        kind: BitwiseOperation,
        is_64_bit: bool,
    },
    Float {
        kind: FloatOperation,
        is_64_bit: bool,
    },
    Cast(NumericConversionKind),
    /// All other instructions not directly defined
    DefaultString(String),
}

impl TryFrom<&Instruction<'_>> for SerializedInstruction {
    type Error = error::WatError;

    fn try_from(value: &Instruction<'_>) -> Result<Self, Self::Error> {
        // TODO: Make this a macro to reduce common patterns
        Ok(match value {
            Instruction::Unreachable
            | Instruction::Nop
            | Instruction::Return
            | Instruction::Drop => Self::Simple(
                try_simple_instruction_from(value)
                    .ok_or(WatError::invalid_instruction("Simple", value))?,
            ),
            Instruction::Block(b) | Instruction::If(b) | Instruction::Loop(b) => Self::Block {
                label: b.label.map(|id| id.name().to_string()).unwrap_or_default(),
                kind: try_block_kind_from(value)
                    .ok_or(WatError::invalid_instruction("Block Kind", value))?,
                inout: Some((&b.ty).try_into()?),
            },
            Instruction::Else(e) | Instruction::End(e) => Self::Block {
                label: e.map(|id| id.name().to_string()).unwrap_or_default(),
                kind: try_block_kind_from(value)
                    .ok_or(WatError::invalid_instruction("Block Kind", value))?,
                inout: None,
            },
            Instruction::Br(i) => Self::Branch {
                default_label: index_to_string(i),
                other_labels: Vec::default(),
                is_conditional: false,
            },
            Instruction::BrIf(i) => Self::Branch {
                default_label: index_to_string(i),
                other_labels: Vec::default(),
                is_conditional: true,
            },
            Instruction::BrTable(br_table) => Self::Branch {
                default_label: index_to_string(&br_table.default),
                other_labels: br_table.labels.iter().map(index_to_string).collect(),
                is_conditional: true,
            },
            Instruction::Call(i) => Self::Call {
                index: index_to_string(i),
                inout: InputOutput::default(),
            },
            Instruction::CallIndirect(ci) => Self::Call {
                index: index_to_string(&ci.table),
                inout: (&ci.ty).try_into()?,
            },
            Instruction::LocalGet(i)
            | Instruction::LocalSet(i)
            | Instruction::LocalTee(i)
            | Instruction::GlobalGet(i)
            | Instruction::GlobalSet(i) => Self::Data {
                kind: try_data_instruction_from(value)
                    .ok_or(WatError::invalid_instruction("Data", value))?,
                location: index_to_string(i),
            },
            Instruction::MemorySize(m) | Instruction::MemoryGrow(m) => Self::Data {
                kind: try_data_instruction_from(value).unwrap(),
                location: index_to_string(&m.mem),
            },
            Instruction::I32Load(m)
            | Instruction::I64Load(m)
            | Instruction::F32Load(m)
            | Instruction::F64Load(m)
            | Instruction::I32Load8s(m)
            | Instruction::I32Load8u(m)
            | Instruction::I32Load16s(m)
            | Instruction::I32Load16u(m)
            | Instruction::I64Load8s(m)
            | Instruction::I64Load8u(m)
            | Instruction::I64Load16s(m)
            | Instruction::I64Load16u(m)
            | Instruction::I64Load32s(m)
            | Instruction::I64Load32u(m) => Self::Memory {
                location: index_to_string(&m.memory),
                typ: data_type_of_instruction(value).unwrap(),
                offset: m.offset as u32,
                alignment: ByteKind::from_alignment(m.align),
                count: try_byte_count_from(value)
                    .ok_or(WatError::invalid_instruction("Memory", value))?,
                is_storing: false,
            },
            Instruction::I32Store(m)
            | Instruction::I64Store(m)
            | Instruction::F32Store(m)
            | Instruction::F64Store(m)
            | Instruction::I32Store8(m)
            | Instruction::I32Store16(m)
            | Instruction::I64Store8(m)
            | Instruction::I64Store16(m)
            | Instruction::I64Store32(m) => Self::Memory {
                location: index_to_string(&m.memory),
                typ: data_type_of_instruction(value).unwrap(),
                offset: m.offset as u32,
                alignment: ByteKind::from_alignment(m.align),
                count: try_byte_count_from(value)
                    .ok_or(WatError::invalid_instruction("Memory", value))?,
                is_storing: true,
            },
            Instruction::I32Const(i) => Self::Const {
                typ: SerializableWatType::I32,
                value: i.into(),
            },
            Instruction::I64Const(i) => Self::Const {
                typ: SerializableWatType::I64,
                value: i.into(),
            },
            Instruction::F32Const(f) => Self::Const {
                typ: SerializableWatType::F32,
                value: f.into(),
            },
            Instruction::F64Const(f) => Self::Const {
                typ: SerializableWatType::I64,
                value: f.into(),
            },
            Instruction::I32Add
            | Instruction::I32Sub
            | Instruction::I32Mul
            | Instruction::I32DivS
            | Instruction::I32DivU
            | Instruction::I32RemS
            | Instruction::I32RemU
            | Instruction::I64Add
            | Instruction::I64Sub
            | Instruction::I64Mul
            | Instruction::I64DivS
            | Instruction::I64DivU
            | Instruction::I64RemS
            | Instruction::I64RemU
            | Instruction::F32Add
            | Instruction::F32Sub
            | Instruction::F32Mul
            | Instruction::F32Div
            | Instruction::F64Add
            | Instruction::F64Sub
            | Instruction::F64Mul
            | Instruction::F64Div => Self::Arithmetic {
                kind: try_arithmetic_from(value)
                    .ok_or(WatError::invalid_instruction("Arithmetic", value))?,
                typ: data_type_of_instruction(value)
                    .ok_or(WatError::invalid_instruction("Numeric", value))?,
            },
            Instruction::I32Eqz
            | Instruction::I32Eq
            | Instruction::I32Ne
            | Instruction::I32LtS
            | Instruction::I32LtU
            | Instruction::I32GtS
            | Instruction::I32GtU
            | Instruction::I32LeS
            | Instruction::I32LeU
            | Instruction::I32GeS
            | Instruction::I32GeU
            | Instruction::I64Eqz
            | Instruction::I64Eq
            | Instruction::I64Ne
            | Instruction::I64LtS
            | Instruction::I64LtU
            | Instruction::I64GtS
            | Instruction::I64GtU
            | Instruction::I64LeS
            | Instruction::I64LeU
            | Instruction::I64GeS
            | Instruction::I64GeU
            | Instruction::F32Eq
            | Instruction::F32Ne
            | Instruction::F32Lt
            | Instruction::F32Gt
            | Instruction::F32Le
            | Instruction::F32Ge
            | Instruction::F64Eq
            | Instruction::F64Ne
            | Instruction::F64Lt
            | Instruction::F64Gt
            | Instruction::F64Le
            | Instruction::F64Ge => Self::Comparison {
                kind: try_comparison_from(value)
                    .ok_or(WatError::invalid_instruction("Comparison", value))?,
                typ: data_type_of_instruction(value)
                    .ok_or(WatError::invalid_instruction("Numeric", value))?,
            },
            Instruction::I32Clz
            | Instruction::I32Ctz
            | Instruction::I32Popcnt
            | Instruction::I64Clz
            | Instruction::I64Ctz
            | Instruction::I64Popcnt
            | Instruction::I32And
            | Instruction::I32Or
            | Instruction::I32Xor
            | Instruction::I32Shl
            | Instruction::I32ShrS
            | Instruction::I32ShrU
            | Instruction::I32Rotl
            | Instruction::I32Rotr
            | Instruction::I64And
            | Instruction::I64Or
            | Instruction::I64Xor
            | Instruction::I64Shl
            | Instruction::I64ShrS
            | Instruction::I64ShrU
            | Instruction::I64Rotl
            | Instruction::I64Rotr => Self::Bitwise {
                kind: try_bitwise_from(value)
                    .ok_or(WatError::invalid_instruction("Bitwise", value))?,
                is_64_bit: is_64_bit_instruction(value)
                    .ok_or(WatError::invalid_instruction("32/64 Bit", value))?,
            },
            Instruction::F32Abs
            | Instruction::F32Neg
            | Instruction::F32Ceil
            | Instruction::F32Floor
            | Instruction::F32Trunc
            | Instruction::F32Nearest
            | Instruction::F32Sqrt
            | Instruction::F32Min
            | Instruction::F32Max
            | Instruction::F32Copysign
            | Instruction::F64Abs
            | Instruction::F64Neg
            | Instruction::F64Ceil
            | Instruction::F64Floor
            | Instruction::F64Trunc
            | Instruction::F64Nearest
            | Instruction::F64Sqrt
            | Instruction::F64Min
            | Instruction::F64Max
            | Instruction::F64Copysign => Self::Float {
                kind: try_float_op_from(value)
                    .ok_or(WatError::invalid_instruction("Floating Point", value))?,
                is_64_bit: is_64_bit_instruction(value)
                    .ok_or(WatError::invalid_instruction("32/64 Bit", value))?,
            },
            Instruction::I32WrapI64
            | Instruction::I32TruncF32S
            | Instruction::I32TruncF32U
            | Instruction::I32TruncF64S
            | Instruction::I32TruncF64U
            | Instruction::I64ExtendI32S
            | Instruction::I64ExtendI32U
            | Instruction::I64TruncF32S
            | Instruction::I64TruncF32U
            | Instruction::I64TruncF64S
            | Instruction::I64TruncF64U
            | Instruction::F32ConvertI32S
            | Instruction::F32ConvertI32U
            | Instruction::F32ConvertI64S
            | Instruction::F32ConvertI64U
            | Instruction::F32DemoteF64
            | Instruction::F64ConvertI32S
            | Instruction::F64ConvertI32U
            | Instruction::F64ConvertI64S
            | Instruction::F64ConvertI64U
            | Instruction::F64PromoteF32
            | Instruction::I32ReinterpretF32
            | Instruction::I64ReinterpretF64
            | Instruction::F32ReinterpretI32
            | Instruction::F64ReinterpretI64 => Self::Cast(
                try_cast_kind_from(value).ok_or(WatError::invalid_instruction("Casting", value))?,
            ),
            other_instruction => Self::DefaultString(format!("{other_instruction:?}")),
        })
    }
}

pub(crate) fn index_to_string(index: &Index) -> String {
    match index {
        Index::Num(idx, _) => idx.to_string(),
        Index::Id(id) => id.name().to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum SerializedInstructionNode {
    NonBlock(SerializedInstruction),
    SingleBlock {
        label: String,
        inout: InputOutput,
        is_loop: bool,
        inner_nodes: Vec<SerializedInstructionNode>,
    },
    ConditionalBlock {
        label: String,
        inout: InputOutput,
        then_nodes: Vec<SerializedInstructionNode>,
        else_nodes: Vec<SerializedInstructionNode>,
    },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkBlock {
    Regular,
    Loop,
    If,
    Else,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct SerializedInstructionTree {
    root: Vec<SerializedInstructionNode>,
}

impl SerializedInstructionTree {
    pub fn get_root(&self) -> &Vec<SerializedInstructionNode> {
        &self.root
    }
}

impl TryFrom<&[Instruction<'_>]> for SerializedInstructionTree {
    type Error = error::WatError;

    fn try_from(value: &[Instruction]) -> Result<Self, Self::Error> {
        let mut wait_stack = Vec::new();
        let mut current = Vec::with_capacity(value.len());
        let mut has_func_ended = false;
        for instruction in value {
            let si = SerializedInstruction::try_from(instruction)?;
            dbg!((&wait_stack, &current, &si));
            match si {
                SerializedInstruction::Block { label, kind, inout } => match kind {
                    BlockKind::Block => {
                        wait_stack.push((current, MarkBlock::Regular, label, inout.unwrap()));
                        current = Vec::new();
                    }
                    BlockKind::Loop => {
                        wait_stack.push((current, MarkBlock::Loop, label, inout.unwrap()));
                        current = Vec::new();
                    }
                    BlockKind::If => {
                        wait_stack.push((current, MarkBlock::If, label, inout.unwrap()));
                        current = Vec::new();
                    }
                    BlockKind::Else => {
                        wait_stack.push((current, MarkBlock::Else, label, InputOutput::default()));
                        current = Vec::new();
                    }
                    BlockKind::End => {
                        let block = wait_stack.pop();

                        if let Some((mut nodes, mark, label, inout)) = block {
                            if mark == MarkBlock::Else {
                                let else_nodes = current;
                                let then_nodes = nodes;
                                let (mut before_if, mark2, label_then, if_inout) = wait_stack
                                    .pop()
                                    .ok_or(WatError::unimplemented_error("No if block provided"))?;
                                if mark2 != MarkBlock::If {
                                    return Err(WatError::unimplemented_error(
                                        "No if block before else provided",
                                    ));
                                }
                                assert!(label_then == label);
                                let if_else_block = SerializedInstructionNode::ConditionalBlock {
                                    label,
                                    inout: if_inout,
                                    then_nodes,
                                    else_nodes,
                                };
                                before_if.push(if_else_block);
                                current = before_if;
                            } else if mark == MarkBlock::If {
                                // If without else
                                let if_block = SerializedInstructionNode::ConditionalBlock {
                                    label,
                                    inout: inout,
                                    then_nodes: current,
                                    else_nodes: Vec::default(),
                                };
                                nodes.push(if_block);
                                current = nodes;
                            } else {
                                let block = SerializedInstructionNode::SingleBlock {
                                    label,
                                    inout,
                                    is_loop: mark == MarkBlock::Loop,
                                    inner_nodes: current,
                                };
                                nodes.push(block);
                                current = nodes;
                            }
                        } else if !has_func_ended {
                            // Function ending End instruction
                            has_func_ended = true;
                            continue;
                        } else {
                            return Err(WatError::invalid_instruction(
                                "No",
                                &Instruction::End(None),
                            ));
                        }
                    }
                },
                _ => current.push(SerializedInstructionNode::NonBlock(si)),
            }
        }
        assert!(wait_stack.is_empty());
        Ok(Self { root: current })
    }
}
