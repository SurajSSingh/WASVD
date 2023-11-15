// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use wast::parser::{self, ParseBuffer};
use wast::Wat;

mod error {
    use serde::Serialize;
    use specta::Type;
    use wast::{token::Span, Error as WastError};

    /// Error Information
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
    pub struct ErrorHolder {
        pub(crate) offset: usize,
        pub(crate) message: String,
    }

    impl From<WastError> for ErrorHolder {
        fn from(value: WastError) -> Self {
            value.span().offset();

            ErrorHolder {
                offset: value.span().offset(),
                message: value.message(),
            }
        }
    }

    /// Create new unimplemented error
    pub fn unimplemented_error(msg: &str) -> WastError {
        WastError::new(Span::from_offset(0), format!("Unimplemented Error: {msg}"))
    }
}

mod interp {
    use crate::error::unimplemented_error;

    use super::error;
    use serde::Deserialize;
    use serde::Serialize;
    use specta::Type;

    use wast::parser::Result as WastResult;
    use wast::Error as WastError;

    use wast::core::ModuleField;
    use wast::token::Id;

    use std::collections::HashMap;

    use wast::core::Func;

    use wast::core::Instruction;

    use wast::token::Index;

    use wast::core::FunctionType;

    use wast;

    use wast::core::ValType;

    /// All Wat types that can be (currently) serialized.
    ///
    /// ## Limitations
    /// All except [ValType::Ref] are supported, but must explicity convert.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    pub enum SerializableWatType {
        I32,
        I64,
        F32,
        F64,
        V128,
    }

    impl<'a> TryFrom<ValType<'a>> for SerializableWatType {
        type Error = WastError;

        /// Try to go from [ValType] to [SerializableWatType]
        fn try_from(value: ValType) -> Result<Self, Self::Error> {
            match value {
                ValType::I32 => Ok(SerializableWatType::I32),
                ValType::I64 => Ok(SerializableWatType::I64),
                ValType::F32 => Ok(SerializableWatType::F32),
                ValType::F64 => Ok(SerializableWatType::F64),
                ValType::V128 => Ok(SerializableWatType::V128),
                ValType::Ref(_) => Err(unimplemented_error("Cannot use Ref type")),
            }
        }
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

    /// The kinds of bits we are observing
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    pub enum BitType {
        LeadingZero,
        TrailingZero,
        NonZero,
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

    /// Bitwise operations
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
    pub enum BitwiseOperation {
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

    /// Kind of numeric operation
    #[derive(
        Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type, derive_more::From,
    )]
    pub enum NumericOperationKind {
        Comparison(ComparisonOperation),
        Arithmetic(ArithmeticOperation),
        Bitwise(BitwiseOperation),
        Float(FloatOperation),
    }

    /// Represents input and output of a block of instructions.
    /// For functions, inputs are parameters and outputs are results.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
    pub struct InputOutput {
        pub(crate) index: Option<String>,
        pub(crate) input: Vec<(Option<String>, SerializableWatType)>,
        pub(crate) output: Vec<SerializableWatType>,
    }

    impl TryFrom<&wast::core::TypeUse<'_, FunctionType<'_>>> for InputOutput {
        type Error = WastError;
        fn try_from(
            value: &wast::core::TypeUse<'_, FunctionType<'_>>,
        ) -> Result<Self, Self::Error> {
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
                        .collect::<Result<_, WastError>>()?,
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

    /// Control flow instructions
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
    pub enum BlockKind {
        #[default]
        Regular,
        If,
        Loop,
    }

    /// Control flow instructions
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
    pub enum ControlFlow {
        Block {
            label: String,
            kind: BlockKind,
            inout: InputOutput,
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
        Else(String),
        End(String),
    }

    /// Serialized instructions based on parts of [Instruction],
    /// but is more generic over types (e.g. a single Add instruction that carries the type).
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
    pub enum SerializedInstruction {
        Unreachable,
        Nop,
        Drop,
        Return,
        ControlFlow(ControlFlow),
        /// Get the value from a local or global identifer or index
        Get {
            loc: String,
            is_local: bool,
        },
        /// Set the value for a local or global identifer or index
        Set {
            loc: String,
            is_local: bool,
        },
        /// Set the value for a local and immediately get put it on the stack
        Tee {
            loc: String,
        },
        /// Load from memory
        Load {
            loc: String,
            offset: u32,
            alignment: ByteKind,
            count: ByteKind,
            typ: SerializableWatType,
            is_signed: bool,
        },
        /// Store to memory
        Store {
            loc: String,
            offset: u32,
            alignment: ByteKind,
            count: ByteKind,
        },
        /// Memory operations
        Memory {
            loc: String,
            will_grow: bool,
        },
        /// Push constant number to stack
        Const {
            typ: SerializableWatType,
            lower32bits: u32,
            upper32bits: u32,
        },
        /// Numeric operation
        NumericOperation {
            typ: SerializableWatType,
            op: NumericOperationKind,
        },
        // Count bits from int
        CountBits {
            bit_to_count: BitType,
            is_64: bool,
        },
        /// Cast from one type to another
        Cast {
            from: SerializableWatType,
            to: SerializableWatType,
            is_signed: bool,
        },
        /// Reinterpret bits of on type to another
        Reinterpret {
            is_int_to_float: bool,
            is_64: bool,
        },
        /// All other instructions not directly defined
        DefaultString(String),
    }

    impl SerializedInstruction {
        pub fn cf_block_instruction(kind: BlockKind, label: String, inout: InputOutput) -> Self {
            Self::ControlFlow(ControlFlow::Block { label, kind, inout })
        }

        pub fn cf_branch_instruction(
            default_label: String,
            other_labels: Option<Vec<String>>,
            is_conditional: bool,
        ) -> Self {
            Self::ControlFlow(ControlFlow::Branch {
                default_label,
                is_conditional,
                other_labels: other_labels.unwrap_or_default(),
            })
        }

        pub fn cf_else_instruction(label: String) -> Self {
            Self::ControlFlow(ControlFlow::Else(label))
        }

        pub fn cf_end_instruction(label: String) -> Self {
            Self::ControlFlow(ControlFlow::End(label))
        }

        pub fn cf_call_instruction(index: String, inout: Option<InputOutput>) -> Self {
            Self::ControlFlow(ControlFlow::Call {
                index,
                inout: inout.unwrap_or_default(),
            })
        }

        pub fn get_instruction(loc: String, is_local: bool) -> Self {
            Self::Get { loc, is_local }
        }

        pub fn set_instruction(loc: String, is_local: bool) -> Self {
            Self::Set { loc, is_local }
        }

        pub fn tee_instruction(loc: String) -> Self {
            Self::Tee { loc }
        }

        pub fn load_instruction(
            typ: SerializableWatType,
            loc: String,
            offset: u64,
            alignment: ByteKind,
            count: Option<ByteKind>,
            is_signed: bool,
        ) -> Self {
            let count = count.unwrap_or(match typ {
                SerializableWatType::I32 | SerializableWatType::F32 => ByteKind::from_byte_count(4),
                SerializableWatType::I64 | SerializableWatType::F64 => ByteKind::from_byte_count(8),
                _ => ByteKind::from_byte_count(16),
            });
            Self::Load {
                loc,
                offset: offset as u32,
                alignment,
                count,
                typ,
                is_signed,
            }
        }
        pub fn store_instruction(
            loc: String,
            offset: u64,
            alignment: ByteKind,
            count: ByteKind,
        ) -> Self {
            Self::Store {
                loc,
                offset: offset as u32,
                alignment,
                count,
            }
        }
        pub fn memory_instruction(loc: String, will_grow: bool) -> Self {
            Self::Memory { loc, will_grow }
        }
        pub fn const_instruction(typ: SerializableWatType, lower: u32, upper: Option<u32>) -> Self {
            Self::Const {
                typ,
                lower32bits: lower,
                upper32bits: upper.unwrap_or(0),
            }
        }
        pub fn count_bits_instruction(bit_to_count: BitType, is_64: bool) -> Self {
            Self::CountBits {
                bit_to_count,
                is_64,
            }
        }
        pub fn comparison_instruction(typ: SerializableWatType, op: ComparisonOperation) -> Self {
            Self::NumericOperation { typ, op: op.into() }
        }
        pub fn arithmetic_instruction(typ: SerializableWatType, op: ArithmeticOperation) -> Self {
            Self::NumericOperation { typ, op: op.into() }
        }
        pub fn bitwise_instruction(typ: SerializableWatType, op: BitwiseOperation) -> Self {
            Self::NumericOperation { typ, op: op.into() }
        }
        pub fn float_op_instruction(op: FloatOperation, is_64: bool) -> Self {
            if is_64 {
                Self::NumericOperation {
                    typ: SerializableWatType::F64,
                    op: op.into(),
                }
            } else {
                Self::NumericOperation {
                    typ: SerializableWatType::F32,
                    op: op.into(),
                }
            }
        }
        pub fn conversion_instruction(
            from: SerializableWatType,
            to: SerializableWatType,
            is_signed: bool,
        ) -> Self {
            Self::Cast {
                from,
                to,
                is_signed,
            }
        }
        pub fn reinterpret_instruction(is_int_to_float: bool, is_64: bool) -> Self {
            Self::Reinterpret {
                is_int_to_float,
                is_64,
            }
        }
    }

    pub(crate) fn index_to_string(index: &Index) -> String {
        match index {
            Index::Num(idx, _) => idx.to_string(),
            Index::Id(id) => id.name().to_string(),
        }
    }

    impl TryFrom<&Instruction<'_>> for SerializedInstruction {
        type Error = WastError;

        fn try_from(value: &Instruction<'_>) -> Result<Self, Self::Error> {
            use ArithmeticOperation as AOp;
            use BitwiseOperation as BOp;
            use ComparisonOperation as COp;
            use FloatOperation as FOp;
            use SerializableWatType as Type;
            use SerializedInstruction as SI;
            // TODO: Make this a macro to reduce common patterns
            Ok(match value {
                Instruction::Block(b) => SI::cf_block_instruction(
                    BlockKind::Regular,
                    b.label.map(|id| id.name().to_string()).unwrap_or_default(),
                    (&b.ty).try_into()?,
                ),
                Instruction::If(b) => SI::cf_block_instruction(
                    BlockKind::If,
                    b.label.map(|id| id.name().to_string()).unwrap_or_default(),
                    (&b.ty).try_into()?,
                ),
                Instruction::Else(e) => {
                    Self::cf_else_instruction(e.map(|id| id.name().to_string()).unwrap_or_default())
                }
                Instruction::Loop(b) => SI::cf_block_instruction(
                    BlockKind::Loop,
                    b.label.map(|id| id.name().to_string()).unwrap_or_default(),
                    (&b.ty).try_into()?,
                ),
                Instruction::End(e) => {
                    Self::cf_end_instruction(e.map(|id| id.name().to_string()).unwrap_or_default())
                }
                Instruction::Unreachable => SI::Unreachable,
                Instruction::Nop => SI::Nop,
                Instruction::Br(i) => SI::cf_branch_instruction(index_to_string(i), None, false),
                Instruction::BrIf(i) => SI::cf_branch_instruction(index_to_string(i), None, true),
                Instruction::BrTable(br_table) => SI::cf_branch_instruction(
                    index_to_string(&br_table.default),
                    Some(br_table.labels.iter().map(|l| index_to_string(l)).collect()),
                    true,
                ),
                Instruction::Return => SI::Return,
                Instruction::Call(i) => SI::cf_call_instruction(index_to_string(i), None),
                Instruction::CallIndirect(ci) => {
                    SI::cf_call_instruction(index_to_string(&ci.table), Some((&ci.ty).try_into()?))
                }
                // Instruction::ReturnCall(i) => todo!(),
                // Instruction::ReturnCallIndirect(ci) => todo!(),
                // Instruction::CallRef(i) => todo!(),
                // Instruction::ReturnCallRef(i) => todo!(),
                // Instruction::FuncBind(fb) => todo!(),
                // Instruction::Let(lt) => todo!(),
                Instruction::Drop => SI::Drop,
                // Instruction::Select(s) => todo!(),
                Instruction::LocalGet(l) => SI::get_instruction(index_to_string(l), true),
                Instruction::LocalSet(l) => SI::set_instruction(index_to_string(l), true),
                Instruction::LocalTee(l) => SI::tee_instruction(index_to_string(l)),
                Instruction::GlobalGet(g) => SI::get_instruction(index_to_string(g), false),
                Instruction::GlobalSet(g) => SI::get_instruction(index_to_string(g), false),
                // Instruction::TableGet(_) => todo!(),
                // Instruction::TableSet(_) => todo!(),
                Instruction::I32Load(m) => SI::load_instruction(
                    Type::I32,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    None,
                    false,
                ),
                Instruction::I64Load(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    None,
                    false,
                ),
                Instruction::F32Load(m) => SI::load_instruction(
                    Type::F32,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    None,
                    false,
                ),
                Instruction::F64Load(m) => SI::load_instruction(
                    Type::F64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    None,
                    false,
                ),
                Instruction::I32Load8s(m) => SI::load_instruction(
                    Type::I32,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(1).into(),
                    true,
                ),
                Instruction::I32Load8u(m) => SI::load_instruction(
                    Type::I32,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(1).into(),
                    false,
                ),
                Instruction::I32Load16s(m) => SI::load_instruction(
                    Type::I32,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(2).into(),
                    true,
                ),
                Instruction::I32Load16u(m) => SI::load_instruction(
                    Type::I32,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(2).into(),
                    false,
                ),
                Instruction::I64Load8s(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(1).into(),
                    true,
                ),
                Instruction::I64Load8u(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(1).into(),
                    false,
                ),
                Instruction::I64Load16s(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(2).into(),
                    true,
                ),
                Instruction::I64Load16u(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(2).into(),
                    false,
                ),
                Instruction::I64Load32s(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(4).into(),
                    true,
                ),
                Instruction::I64Load32u(m) => SI::load_instruction(
                    Type::I64,
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::from_byte_count(4).into(),
                    false,
                ),
                Instruction::I32Store(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits32,
                ),
                Instruction::I64Store(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits64,
                ),
                Instruction::F32Store(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits32,
                ),
                Instruction::F64Store(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits64,
                ),
                Instruction::I32Store8(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits8,
                ),
                Instruction::I32Store16(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits16,
                ),
                Instruction::I64Store8(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits8,
                ),
                Instruction::I64Store16(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits16,
                ),
                Instruction::I64Store32(m) => SI::store_instruction(
                    index_to_string(&m.memory),
                    m.offset,
                    ByteKind::from_alignment(m.align),
                    ByteKind::Bits32,
                ),
                Instruction::MemorySize(m) => {
                    SI::memory_instruction(index_to_string(&m.mem), false)
                }
                Instruction::MemoryGrow(m) => SI::memory_instruction(index_to_string(&m.mem), true),
                // Instruction::MemoryInit(_) => todo!(),
                // Instruction::MemoryCopy(_) => todo!(),
                // Instruction::MemoryFill(_) => todo!(),
                // Instruction::MemoryDiscard(_) => todo!(),
                // Instruction::DataDrop(_) => todo!(),
                // Instruction::ElemDrop(_) => todo!(),
                // Instruction::TableInit(_) => todo!(),
                // Instruction::TableCopy(_) => todo!(),
                // Instruction::TableFill(_) => todo!(),
                // Instruction::TableSize(_) => todo!(),
                // Instruction::TableGrow(_) => todo!(),
                // Instruction::RefNull(_) => todo!(),
                // Instruction::RefIsNull => todo!(),
                // Instruction::RefFunc(_) => todo!(),
                // Instruction::RefAsNonNull => todo!(),
                // Instruction::BrOnNull(_) => todo!(),
                // Instruction::BrOnNonNull(_) => todo!(),
                // Instruction::RefEq => todo!(),
                // Instruction::StructNew(_) => todo!(),
                // Instruction::StructNewDefault(_) => todo!(),
                // Instruction::StructGet(_) => todo!(),
                // Instruction::StructGetS(_) => todo!(),
                // Instruction::StructGetU(_) => todo!(),
                // Instruction::StructSet(_) => todo!(),
                // Instruction::ArrayNew(_) => todo!(),
                // Instruction::ArrayNewDefault(_) => todo!(),
                // Instruction::ArrayNewFixed(_) => todo!(),
                // Instruction::ArrayNewData(_) => todo!(),
                // Instruction::ArrayNewElem(_) => todo!(),
                // Instruction::ArrayGet(_) => todo!(),
                // Instruction::ArrayGetS(_) => todo!(),
                // Instruction::ArrayGetU(_) => todo!(),
                // Instruction::ArraySet(_) => todo!(),
                // Instruction::ArrayLen => todo!(),
                // Instruction::ArrayFill(_) => todo!(),
                // Instruction::ArrayCopy(_) => todo!(),
                // Instruction::ArrayInitData(_) => todo!(),
                // Instruction::ArrayInitElem(_) => todo!(),
                // Instruction::RefI31 => todo!(),
                // Instruction::I31GetS => todo!(),
                // Instruction::I31GetU => todo!(),
                // Instruction::RefTest(_) => todo!(),
                // Instruction::RefCast(_) => todo!(),
                // Instruction::BrOnCast(_) => todo!(),
                // Instruction::BrOnCastFail(_) => todo!(),
                // Instruction::ExternInternalize => todo!(),
                // Instruction::ExternExternalize => todo!(),
                Instruction::I32Const(i) => SI::const_instruction(
                    SerializableWatType::I32,
                    u32::from_ne_bytes(i.to_ne_bytes()),
                    None,
                ),
                Instruction::I64Const(i) => {
                    let bytes = i.to_ne_bytes();
                    let (lower_bytes, upper_bytes) = (
                        [bytes[0], bytes[1], bytes[2], bytes[3]],
                        [bytes[4], bytes[5], bytes[6], bytes[7]],
                    );
                    SI::const_instruction(
                        SerializableWatType::I64,
                        u32::from_ne_bytes(lower_bytes),
                        Some(u32::from_ne_bytes(upper_bytes)),
                    )
                }
                Instruction::F32Const(f) => SI::const_instruction(
                    SerializableWatType::F32,
                    u32::from_ne_bytes(f.bits.to_ne_bytes()),
                    None,
                ),
                Instruction::F64Const(f) => {
                    let bytes = f.bits.to_ne_bytes();
                    let (lower_bytes, upper_bytes) = (
                        [bytes[0], bytes[1], bytes[2], bytes[3]],
                        [bytes[4], bytes[5], bytes[6], bytes[7]],
                    );
                    SI::const_instruction(
                        SerializableWatType::I64,
                        u32::from_ne_bytes(lower_bytes),
                        Some(u32::from_ne_bytes(upper_bytes)),
                    )
                }
                Instruction::I32Clz => SI::count_bits_instruction(BitType::LeadingZero, false),
                Instruction::I32Ctz => SI::count_bits_instruction(BitType::TrailingZero, false),
                Instruction::I32Popcnt => SI::count_bits_instruction(BitType::NonZero, false),
                Instruction::I32Add => SI::arithmetic_instruction(Type::I32, AOp::Addition),
                Instruction::I32Sub => SI::arithmetic_instruction(Type::I32, AOp::Subtraction),
                Instruction::I32Mul => SI::arithmetic_instruction(Type::I32, AOp::Multiplication),
                Instruction::I32DivS => SI::arithmetic_instruction(Type::I32, AOp::DivisonSigned),
                Instruction::I32DivU => SI::arithmetic_instruction(Type::I32, AOp::DivisonUnsigned),
                Instruction::I32RemS => SI::arithmetic_instruction(Type::I32, AOp::RemainderSigned),
                Instruction::I32RemU => {
                    SI::arithmetic_instruction(Type::I32, AOp::RemainderUnsigned)
                }
                Instruction::I32And => SI::bitwise_instruction(Type::I32, BOp::And),
                Instruction::I32Or => SI::bitwise_instruction(Type::I32, BOp::Or),
                Instruction::I32Xor => SI::bitwise_instruction(Type::I32, BOp::Xor),
                Instruction::I32Shl => SI::bitwise_instruction(Type::I32, BOp::ShiftLeft),
                Instruction::I32ShrS => SI::bitwise_instruction(Type::I32, BOp::ShiftRightSigned),
                Instruction::I32ShrU => SI::bitwise_instruction(Type::I32, BOp::ShiftRightUnsigned),
                Instruction::I32Rotl => SI::bitwise_instruction(Type::I32, BOp::RotateLeft),
                Instruction::I32Rotr => SI::bitwise_instruction(Type::I32, BOp::RotateRight),
                Instruction::I64Clz => SI::count_bits_instruction(BitType::LeadingZero, true),
                Instruction::I64Ctz => SI::count_bits_instruction(BitType::TrailingZero, true),
                Instruction::I64Popcnt => SI::count_bits_instruction(BitType::NonZero, true),
                Instruction::I64Add => SI::arithmetic_instruction(Type::I64, AOp::Addition),
                Instruction::I64Sub => SI::arithmetic_instruction(Type::I64, AOp::Subtraction),
                Instruction::I64Mul => SI::arithmetic_instruction(Type::I64, AOp::Multiplication),
                Instruction::I64DivS => SI::arithmetic_instruction(Type::I64, AOp::DivisonSigned),
                Instruction::I64DivU => SI::arithmetic_instruction(Type::I64, AOp::DivisonUnsigned),
                Instruction::I64RemS => SI::arithmetic_instruction(Type::I64, AOp::RemainderSigned),
                Instruction::I64RemU => {
                    SI::arithmetic_instruction(Type::I64, AOp::RemainderUnsigned)
                }
                Instruction::I64And => SI::bitwise_instruction(Type::I64, BOp::And),
                Instruction::I64Or => SI::bitwise_instruction(Type::I64, BOp::Or),
                Instruction::I64Xor => SI::bitwise_instruction(Type::I64, BOp::Xor),
                Instruction::I64Shl => SI::bitwise_instruction(Type::I64, BOp::ShiftLeft),
                Instruction::I64ShrS => SI::bitwise_instruction(Type::I64, BOp::ShiftRightSigned),
                Instruction::I64ShrU => SI::bitwise_instruction(Type::I64, BOp::ShiftRightUnsigned),
                Instruction::I64Rotl => SI::bitwise_instruction(Type::I64, BOp::RotateLeft),
                Instruction::I64Rotr => SI::bitwise_instruction(Type::I64, BOp::RotateRight),
                Instruction::F32Abs => SI::float_op_instruction(FOp::AbsoluteValue, false),
                Instruction::F32Neg => SI::float_op_instruction(FOp::Negation, false),
                Instruction::F32Ceil => SI::float_op_instruction(FOp::Ceiling, false),
                Instruction::F32Floor => SI::float_op_instruction(FOp::Floor, false),
                Instruction::F32Trunc => SI::float_op_instruction(FOp::Truncate, false),
                Instruction::F32Nearest => SI::float_op_instruction(FOp::Nearest, false),
                Instruction::F32Sqrt => SI::float_op_instruction(FOp::SquareRoot, false),
                Instruction::F32Add => SI::arithmetic_instruction(Type::F32, AOp::Addition),
                Instruction::F32Sub => SI::arithmetic_instruction(Type::F32, AOp::Subtraction),
                Instruction::F32Mul => SI::arithmetic_instruction(Type::F32, AOp::Multiplication),
                Instruction::F32Div => SI::arithmetic_instruction(Type::F32, AOp::DivisonSigned),
                Instruction::F32Min => SI::float_op_instruction(FOp::Minimum, false),
                Instruction::F32Max => SI::float_op_instruction(FOp::Maximum, false),
                Instruction::F32Copysign => SI::float_op_instruction(FOp::CopySign, false),
                Instruction::F64Abs => SI::float_op_instruction(FOp::AbsoluteValue, true),
                Instruction::F64Neg => SI::float_op_instruction(FOp::Negation, true),
                Instruction::F64Ceil => SI::float_op_instruction(FOp::Ceiling, true),
                Instruction::F64Floor => SI::float_op_instruction(FOp::Floor, true),
                Instruction::F64Trunc => SI::float_op_instruction(FOp::Truncate, true),
                Instruction::F64Nearest => SI::float_op_instruction(FOp::Nearest, true),
                Instruction::F64Sqrt => SI::float_op_instruction(FOp::SquareRoot, true),
                Instruction::F64Add => SI::arithmetic_instruction(Type::F32, AOp::Addition),
                Instruction::F64Sub => SI::arithmetic_instruction(Type::F32, AOp::Subtraction),
                Instruction::F64Mul => SI::arithmetic_instruction(Type::F32, AOp::Multiplication),
                Instruction::F64Div => SI::arithmetic_instruction(Type::F32, AOp::DivisonSigned),
                Instruction::F64Min => SI::float_op_instruction(FOp::Minimum, true),
                Instruction::F64Max => SI::float_op_instruction(FOp::Maximum, true),
                Instruction::F64Copysign => SI::float_op_instruction(FOp::CopySign, true),
                Instruction::I32Eqz => SI::comparison_instruction(Type::I32, COp::EqualZero),
                Instruction::I32Eq => SI::comparison_instruction(Type::I32, COp::Equal),
                Instruction::I32Ne => SI::comparison_instruction(Type::I32, COp::NotEqual),
                Instruction::I32LtS => SI::comparison_instruction(Type::I32, COp::LessThenSigned),
                Instruction::I32LtU => SI::comparison_instruction(Type::I32, COp::LessThenUnsigned),
                Instruction::I32GtS => {
                    SI::comparison_instruction(Type::I32, COp::GreaterThenSigned)
                }
                Instruction::I32GtU => {
                    SI::comparison_instruction(Type::I32, COp::GreaterThenUnsigned)
                }
                Instruction::I32LeS => {
                    SI::comparison_instruction(Type::I32, COp::LessThenOrEqualToSigned)
                }
                Instruction::I32LeU => {
                    SI::comparison_instruction(Type::I32, COp::LessThenOrEqualToUnsigned)
                }
                Instruction::I32GeS => {
                    SI::comparison_instruction(Type::I32, COp::GreaterThenOrEqualToSigned)
                }
                Instruction::I32GeU => {
                    SI::comparison_instruction(Type::I32, COp::GreaterThenOrEqualToUnsigned)
                }
                Instruction::I64Eqz => SI::comparison_instruction(Type::I64, COp::EqualZero),
                Instruction::I64Eq => SI::comparison_instruction(Type::I64, COp::Equal),
                Instruction::I64Ne => SI::comparison_instruction(Type::I64, COp::NotEqual),
                Instruction::I64LtS => SI::comparison_instruction(Type::I64, COp::LessThenSigned),
                Instruction::I64LtU => SI::comparison_instruction(Type::I64, COp::LessThenUnsigned),
                Instruction::I64GtS => {
                    SI::comparison_instruction(Type::I64, COp::GreaterThenSigned)
                }
                Instruction::I64GtU => {
                    SI::comparison_instruction(Type::I64, COp::GreaterThenUnsigned)
                }
                Instruction::I64LeS => {
                    SI::comparison_instruction(Type::I64, COp::LessThenOrEqualToSigned)
                }
                Instruction::I64LeU => {
                    SI::comparison_instruction(Type::I64, COp::LessThenOrEqualToUnsigned)
                }
                Instruction::I64GeS => {
                    SI::comparison_instruction(Type::I64, COp::GreaterThenOrEqualToSigned)
                }
                Instruction::I64GeU => {
                    SI::comparison_instruction(Type::I64, COp::GreaterThenOrEqualToUnsigned)
                }
                Instruction::F32Eq => SI::comparison_instruction(Type::F32, COp::Equal),
                Instruction::F32Ne => SI::comparison_instruction(Type::F32, COp::NotEqual),
                Instruction::F32Lt => SI::comparison_instruction(Type::F32, COp::LessThenSigned),
                Instruction::F32Gt => SI::comparison_instruction(Type::F32, COp::GreaterThenSigned),
                Instruction::F32Le => {
                    SI::comparison_instruction(Type::F32, COp::LessThenOrEqualToSigned)
                }
                Instruction::F32Ge => {
                    SI::comparison_instruction(Type::F32, COp::GreaterThenOrEqualToSigned)
                }
                Instruction::F64Eq => SI::comparison_instruction(Type::F32, COp::Equal),
                Instruction::F64Ne => SI::comparison_instruction(Type::F32, COp::NotEqual),
                Instruction::F64Lt => SI::comparison_instruction(Type::F32, COp::LessThenSigned),
                Instruction::F64Gt => SI::comparison_instruction(Type::F32, COp::GreaterThenSigned),
                Instruction::F64Le => {
                    SI::comparison_instruction(Type::F32, COp::LessThenOrEqualToSigned)
                }
                Instruction::F64Ge => {
                    SI::comparison_instruction(Type::F32, COp::GreaterThenOrEqualToSigned)
                }
                Instruction::I32WrapI64 => SI::conversion_instruction(Type::I64, Type::I32, false),
                Instruction::I32TruncF32S => SI::conversion_instruction(Type::F32, Type::I32, true),
                Instruction::I32TruncF32U => {
                    SI::conversion_instruction(Type::F32, Type::I32, false)
                }
                Instruction::I32TruncF64S => SI::conversion_instruction(Type::F64, Type::I32, true),
                Instruction::I32TruncF64U => {
                    SI::conversion_instruction(Type::F64, Type::I32, false)
                }
                Instruction::I64ExtendI32S => {
                    SI::conversion_instruction(Type::I32, Type::I64, true)
                }
                Instruction::I64ExtendI32U => {
                    SI::conversion_instruction(Type::I32, Type::I64, false)
                }
                Instruction::I64TruncF32S => SI::conversion_instruction(Type::F32, Type::I64, true),
                Instruction::I64TruncF32U => {
                    SI::conversion_instruction(Type::F32, Type::I64, false)
                }
                Instruction::I64TruncF64S => SI::conversion_instruction(Type::F64, Type::I64, true),
                Instruction::I64TruncF64U => {
                    SI::conversion_instruction(Type::F64, Type::I64, false)
                }
                Instruction::F32ConvertI32S => {
                    SI::conversion_instruction(Type::I32, Type::F32, true)
                }
                Instruction::F32ConvertI32U => {
                    SI::conversion_instruction(Type::I32, Type::F32, false)
                }
                Instruction::F32ConvertI64S => {
                    SI::conversion_instruction(Type::I64, Type::F32, true)
                }
                Instruction::F32ConvertI64U => {
                    SI::conversion_instruction(Type::I64, Type::F32, false)
                }
                Instruction::F32DemoteF64 => SI::conversion_instruction(Type::F64, Type::F32, true),
                Instruction::F64ConvertI32S => {
                    SI::conversion_instruction(Type::I32, Type::F64, true)
                }
                Instruction::F64ConvertI32U => {
                    SI::conversion_instruction(Type::I32, Type::F64, false)
                }
                Instruction::F64ConvertI64S => {
                    SI::conversion_instruction(Type::I64, Type::F64, true)
                }
                Instruction::F64ConvertI64U => {
                    SI::conversion_instruction(Type::I64, Type::F64, true)
                }
                Instruction::F64PromoteF32 => {
                    SI::conversion_instruction(Type::F32, Type::F64, false)
                }
                Instruction::I32ReinterpretF32 => SI::reinterpret_instruction(false, false),
                Instruction::I64ReinterpretF64 => SI::reinterpret_instruction(false, true),
                Instruction::F32ReinterpretI32 => SI::reinterpret_instruction(true, false),
                Instruction::F64ReinterpretI64 => SI::reinterpret_instruction(true, true),
                // Instruction::I32TruncSatF32S => todo!(),
                // Instruction::I32TruncSatF32U => todo!(),
                // Instruction::I32TruncSatF64S => todo!(),
                // Instruction::I32TruncSatF64U => todo!(),
                // Instruction::I64TruncSatF32S => todo!(),
                // Instruction::I64TruncSatF32U => todo!(),
                // Instruction::I64TruncSatF64S => todo!(),
                // Instruction::I64TruncSatF64U => todo!(),
                // Instruction::I32Extend8S => todo!(),
                // Instruction::I32Extend16S => todo!(),
                // Instruction::I64Extend8S => todo!(),
                // Instruction::I64Extend16S => todo!(),
                // Instruction::I64Extend32S => todo!(),
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
                // Instruction::Try(_) => todo!(),
                // Instruction::Catch(_) => todo!(),
                // Instruction::Throw(_) => todo!(),
                // Instruction::Rethrow(_) => todo!(),
                // Instruction::Delegate(_) => todo!(),
                // Instruction::CatchAll => todo!(),
                // Instruction::ThrowRef => todo!(),
                // Instruction::TryTable(_) => todo!(),
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
                other_instruction => Self::DefaultString(format!("{other_instruction:?}")),
            })
        }
    }

    /// A basic Wa(s)t Function
    ///
    /// ## Note:
    /// Does not work with imported functions, as it assumes nothing about other modules
    #[derive(Debug, Clone, Serialize, Type)]
    pub struct WastFunc {
        pub(crate) name: Option<String>,
        pub(crate) parameters: Vec<(Option<String>, SerializableWatType)>,
        pub(crate) locals: Vec<(Option<String>, SerializableWatType)>,
        pub(crate) body: Vec<SerializedInstruction>,
        pub(crate) result: Vec<SerializableWatType>,
    }

    impl WastFunc {
        pub fn set_name_from_number(&mut self, index: usize) {
            self.name = Some(index.to_string());
        }
    }

    impl TryFrom<&Func<'_>> for WastFunc {
        type Error = WastError;

        fn try_from(value: &Func<'_>) -> Result<Self, Self::Error> {
            let name = value.id.map(|i| i.name().to_string());
            if value.ty.index.is_some() {
                return Err(unimplemented_error(
                    "Index value should not be assigned for functions, I believe",
                ));
            }
            let (parameters, result) = match &value.ty.inline {
                Some(FunctionType { params, results }) => (
                    params
                        .iter()
                        .map(|p| match SerializableWatType::try_from(p.2) {
                            Ok(ty) => Ok((p.0.map(|i| i.name().to_string()), ty)),
                            Err(err) => Err(err),
                        })
                        .collect::<Result<Vec<_>, WastError>>()?,
                    results
                        .iter()
                        // TODO: Remove clone for r
                        .map(|r| SerializableWatType::try_from(*r))
                        .collect::<Result<Vec<_>, WastError>>()?,
                ),
                None => (Vec::default(), Vec::default()),
            };
            match &value.kind {
                wast::core::FuncKind::Import(_) => {
                    Err(unimplemented_error("Import functions are not supported"))
                }
                wast::core::FuncKind::Inline { locals, expression } => Ok(WastFunc {
                    name,
                    parameters,
                    locals: locals
                        .iter()
                        .map(|l| match SerializableWatType::try_from(l.ty) {
                            Ok(ty) => Ok((l.id.map(|i| i.name().to_string()), ty)),
                            Err(err) => Err(err),
                        })
                        .collect::<Result<Vec<_>, WastError>>()?,
                    body: expression
                        .instrs
                        .iter()
                        .map(|ins| SerializedInstruction::try_from(ins))
                        .collect::<Result<_, WastError>>()?,
                    result,
                }),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type)]
    pub enum NumLocationKind {
        Function,
        Global,
        Memory,
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
    pub struct GlobalData {
        name: String,
        typ: SerializableWatType,
        is_mutable: bool,
        val: Vec<SerializedInstruction>,
    }

    impl GlobalData {
        pub fn new(
            name: String,
            typ: SerializableWatType,
            is_mutable: bool,
            val: Vec<SerializedInstruction>,
        ) -> Self {
            Self {
                name,
                typ,
                is_mutable,
                val,
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
    pub struct MemoryData {
        name: String,
        min_lower: u32,
        min_upper: u32,
        max_lower: u32,
        max_upper: u32,
        is_32: bool,
        is_shared: bool,
        data: Vec<u8>,
    }

    impl MemoryData {
        pub fn new(
            name: String,
            min: u64,
            max: Option<u64>,
            is_32: bool,
            is_shared: bool,
            data: Vec<u8>,
        ) -> Self {
            let min_bytes = min.to_ne_bytes();
            let max_bytes = max.unwrap_or(u32::MAX as u64).to_ne_bytes();
            let min_lower =
                u32::from_ne_bytes([min_bytes[0], min_bytes[1], min_bytes[2], min_bytes[3]]);
            let min_upper =
                u32::from_ne_bytes([min_bytes[4], min_bytes[5], min_bytes[6], min_bytes[7]]);
            let max_lower =
                u32::from_ne_bytes([max_bytes[0], max_bytes[1], max_bytes[2], max_bytes[3]]);
            let max_upper =
                u32::from_ne_bytes([max_bytes[4], max_bytes[5], max_bytes[6], max_bytes[7]]);
            Self {
                name,
                min_lower,
                min_upper,
                max_lower,
                max_upper,
                is_32,
                is_shared,
                data,
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Type)]
    pub struct InterpreterStructure {
        pub(crate) name: String,
        pub(crate) exported: HashMap<String, (NumLocationKind, u32)>,
        pub(crate) globals: Vec<GlobalData>,
        pub(crate) memory: Vec<MemoryData>,
        pub(crate) func: Vec<WastFunc>,
    }

    impl InterpreterStructure {
        pub fn try_new(_text: &str, fields: &[ModuleField], name: &Option<Id>) -> WastResult<Self> {
            let mut exported = HashMap::new();
            let mut globals = Vec::new();
            let mut memory = Vec::new();
            let mut func = Vec::new();
            // let mut start = 0;
            for (i, field) in fields.iter().enumerate() {
                match field {
                    ModuleField::Type(_) => todo!("Type field not implemented"),
                    ModuleField::Rec(_) => todo!("Rec field not implemented"),
                    ModuleField::Import(_) => todo!("Import field not implemented"),
                    ModuleField::Func(f) => {
                        exported.extend(f.exports.names.iter().map(|name| {
                            (
                                name.to_string(),
                                (NumLocationKind::Function, func.len() as u32),
                            )
                        }));
                        let mut function = WastFunc::try_from(f)?;
                        if function.name.is_none() {
                            function.set_name_from_number(func.len())
                        };
                        func.push(function);
                    }
                    ModuleField::Table(_) => todo!("Table field not implemented"),
                    ModuleField::Memory(m) => {
                        exported.extend(m.exports.names.iter().map(|name| {
                            (
                                name.to_string(),
                                (NumLocationKind::Memory, func.len() as u32),
                            )
                        }));
                        match &m.kind {
                            wast::core::MemoryKind::Import { import: _, ty: _ } => {
                                Err(unimplemented_error("Imported memory not yet implemented."))?
                            }
                            wast::core::MemoryKind::Normal(mt) => match mt {
                                wast::core::MemoryType::B32 { limits, shared } => {
                                    memory.push(MemoryData::new(
                                        m.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                        limits.min as u64,
                                        limits.max.map(|n| n as u64),
                                        true,
                                        *shared,
                                        Vec::new(),
                                    ));
                                }
                                wast::core::MemoryType::B64 { limits, shared } => {
                                    memory.push(MemoryData::new(
                                        m.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                        limits.min,
                                        limits.max,
                                        false,
                                        *shared,
                                        Vec::new(),
                                    ));
                                }
                            },
                            wast::core::MemoryKind::Inline { is_32: _, data: _ } => {
                                Err(unimplemented_error("Inline memory not yet implemented."))?
                            }
                        }
                    }
                    ModuleField::Global(g) => {
                        exported.extend(g.exports.names.iter().map(|name| {
                            (
                                name.to_string(),
                                (NumLocationKind::Global, func.len() as u32),
                            )
                        }));
                        match &g.kind {
                            wast::core::GlobalKind::Import(_) => {
                                Err(unimplemented_error("Imported globals not yet implemented."))?
                            }
                            wast::core::GlobalKind::Inline(e) => {
                                globals.push(GlobalData::new(
                                    g.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                    g.ty.ty.try_into()?,
                                    g.ty.mutable,
                                    e.instrs
                                        .iter()
                                        .map(|ins| ins.try_into())
                                        .collect::<Result<_, _>>()?,
                                ));
                            }
                        }
                    }
                    ModuleField::Export(e) => match dbg!(e).kind {
                        wast::core::ExportKind::Func => {
                            for (i, f) in func.iter().enumerate() {
                                if f.name
                                    .as_ref()
                                    .is_some_and(|item| item == &index_to_string(&e.item))
                                {
                                    exported.insert(
                                        e.name.to_string(),
                                        (NumLocationKind::Function, i as u32),
                                    );
                                    break;
                                }
                            }
                        }
                        wast::core::ExportKind::Table => todo!("Export Tables not implemented"),
                        wast::core::ExportKind::Memory => {
                            for (i, m) in memory.iter().enumerate() {
                                if m.name == index_to_string(&e.item) {
                                    exported.insert(
                                        e.name.to_string(),
                                        (NumLocationKind::Memory, i as u32),
                                    );
                                    break;
                                }
                            }
                        }
                        wast::core::ExportKind::Global => {
                            for (i, g) in globals.iter().enumerate() {
                                if g.name == index_to_string(&e.item) {
                                    exported.insert(
                                        e.name.to_string(),
                                        (NumLocationKind::Global, i as u32),
                                    );
                                    break;
                                }
                            }
                        }
                        wast::core::ExportKind::Tag => todo!("Export Tags not implemented"),
                    },
                    ModuleField::Start(_) => todo!("Start field not implemented"),
                    ModuleField::Elem(_) => todo!("Element field not implemented"),
                    ModuleField::Data(_) => todo!("Data field not implemented"),
                    ModuleField::Tag(_) => todo!("Tag field not implemented"),
                    ModuleField::Custom(_) => todo!("Custom field not implemented"),
                }
            }
            Ok(InterpreterStructure {
                name: name.map(|id| id.name().to_string()).unwrap_or_default(),
                exported,
                globals,
                memory,
                func,
            })
        }
    }
}

fn create_buffer(text: &str) -> parser::Result<ParseBuffer> {
    ParseBuffer::new(text)
}

fn convert_buffer<'a>(buffer: &'a ParseBuffer<'a>) -> parser::Result<Wat<'a>> {
    parser::parse::<'a, Wat<'a>>(buffer)
}

#[tauri::command]
#[specta::specta]
fn transform(text: &str) -> Result<interp::InterpreterStructure, error::ErrorHolder> {
    let buf = create_buffer(text)?;
    let module = convert_buffer(&buf)?;
    match module {
        Wat::Module(m) => match &m.kind {
            wast::core::ModuleKind::Text(fields) => {
                Ok(interp::InterpreterStructure::try_new(text, fields, &m.id)?)
            }
            wast::core::ModuleKind::Binary(_) => Err(error::unimplemented_error(
                "Unimplemented Error: Cannot binary type currently.",
            )
            .into()),
        },
        Wat::Component(_) => {
            Err(error::unimplemented_error("Cannot compile components currently.").into())
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![transform])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod export_bindings {
    //! This module is only for exporting binding for TypeScript
    use super::*;
    #[test]
    fn export_bindings() {
        dbg!(tauri_specta::ts::export(
            specta::collect_types![transform],
            "../src/lib/bindings.ts"
        ))
        .unwrap();
    }
}
