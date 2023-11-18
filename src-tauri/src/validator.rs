use std::collections::{HashMap, HashSet};

use crate::{
    error::{WatError, WatResult},
    instruction::SerializedInstruction,
    marker::{SerializableWatType, SimpleInstruction},
};

/// A simple Wat validator, checking both stack is correctly sized and has correct type at each instruction
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Validator {
    stack: Vec<SerializableWatType>,
    final_stack: Vec<SerializableWatType>,
    globals: HashMap<String, (bool, SerializableWatType)>,
    locals: HashMap<String, SerializableWatType>,
    memory_names: HashSet<String>,
    functions: HashMap<String, (Vec<SerializableWatType>, Vec<SerializableWatType>)>,
}

impl Validator {
    pub fn new(
        globals: HashMap<String, (bool, SerializableWatType)>,
        locals: HashMap<String, SerializableWatType>,
        functions: HashMap<String, (Vec<SerializableWatType>, Vec<SerializableWatType>)>,
        memory_names: HashSet<String>,
        final_stack: Vec<SerializableWatType>,
    ) -> Self {
        Validator {
            stack: Vec::new(),
            final_stack,
            globals,
            locals,
            memory_names,
            functions,
        }
    }

    pub fn process(&mut self, instruction: &SerializedInstruction) -> WatResult<()> {
        match instruction {
            SerializedInstruction::Simple(s) => match s {
                SimpleInstruction::Unreachable => Ok(()),
                SimpleInstruction::Nop => Ok(()),
                SimpleInstruction::Drop => match self.stack.pop() {
                    Some(_) => Ok(()),
                    None => Err(WatError::empty_stack(1)),
                },
                SimpleInstruction::Return => {
                    if self.stack != self.final_stack {
                        Err(WatError::mismatched_inout(
                            &self.final_stack,
                            &self.stack,
                            true,
                        ))
                    } else {
                        Ok(())
                    }
                }
            },
            SerializedInstruction::Block { label, kind, inout } => todo!(),
            SerializedInstruction::Branch {
                default_label,
                other_labels,
                is_conditional,
            } => todo!(),
            SerializedInstruction::Call { inout, index } => {
                // Verify function exists and has correct tyle
                if let Some((params, results)) = self.functions.get(index) {
                    let actual_param = inout.input.iter().map(|(_, t)| *t).collect::<Vec<_>>();
                    if params.len() != actual_param.len()
                        || params.iter().zip(actual_param.iter()).any(|(e, a)| e != a)
                    {
                        return Err(WatError::mismatched_inout(params, &actual_param, false));
                    } else if results.len() != inout.output.len()
                        || results.iter().zip(inout.output.iter()).any(|(e, a)| e != a)
                    {
                        return Err(WatError::mismatched_inout(results, &inout.output, true));
                    }
                } else {
                    return Err(WatError::name_resolution_error(
                        index.clone(),
                        crate::NumLocationKind::Function,
                    ));
                }
                // Verify stack is correct input
                inout.input.iter().map(|(_, t)| t).try_rfold(
                    0u32,
                    |current, expected| match self.stack.pop() {
                        Some(actual) if actual == *expected => Ok(current + 1),
                        Some(actual) => Err(WatError::type_error(expected, &actual)),
                        None => Err(WatError::not_enough_on_stack(
                            inout.input.len(),
                            current as usize,
                        )),
                    },
                )?;
                // Push results to stack
                Ok(self.stack.extend(inout.output.clone()))
            }
            SerializedInstruction::Data { kind, location } => match kind {
                crate::marker::DataInstruction::GetLocal => {
                    if let Some(t) = self.locals.get(location) {
                        Ok(self.stack.push(*t))
                    } else {
                        Err(WatError::local_resolution_error(location.clone()))
                    }
                }
                crate::marker::DataInstruction::GetGlobal => {
                    if let Some((_, t)) = self.globals.get(location) {
                        Ok(self.stack.push(*t))
                    } else {
                        Err(WatError::name_resolution_error(
                            location.clone(),
                            crate::NumLocationKind::Global,
                        ))
                    }
                }
                crate::marker::DataInstruction::SetLocal => {
                    match (self.locals.get(location), self.stack.pop()) {
                        (Some(expected), Some(actual)) if expected == &actual => Ok(()),
                        (Some(expected), Some(actual)) => {
                            Err(WatError::type_error(expected, &actual))
                        }
                        (Some(_), None) => Err(WatError::empty_stack(1)),
                        (None, _) => Err(WatError::local_resolution_error(location.clone())),
                    }
                }
                crate::marker::DataInstruction::SetGlobal => {
                    match (self.globals.get(location), self.stack.pop()) {
                        (Some((is_mut, expected)), Some(actual))
                            if expected == &actual && *is_mut =>
                        {
                            Ok(())
                        }
                        (Some((is_mut, expected)), Some(actual)) => {
                            if !is_mut {
                                Err(WatError::setting_immutable_global_error(location))
                            } else {
                                Err(WatError::type_error(expected, &actual))
                            }
                        }
                        (Some(_), None) => Err(WatError::empty_stack(1)),
                        (None, _) => Err(WatError::local_resolution_error(location.clone())),
                    }
                }
                crate::marker::DataInstruction::TeeLocal => {
                    match (self.locals.get(location), self.stack.pop()) {
                        (Some(expected), Some(actual)) if expected == &actual => {
                            Ok(self.stack.push(actual))
                        }
                        (Some(expected), Some(actual)) => {
                            Err(WatError::type_error(expected, &actual))
                        }
                        (Some(_), None) => Err(WatError::empty_stack(1)),
                        (None, _) => Err(WatError::local_resolution_error(location.clone())),
                    }
                }
                crate::marker::DataInstruction::GetMemorySize => {
                    if self.memory_names.contains(location) {
                        Ok(self.stack.push(SerializableWatType::I32))
                    } else {
                        Err(WatError::name_resolution_error(
                            location.clone(),
                            crate::NumLocationKind::Memory,
                        ))
                    }
                }
                crate::marker::DataInstruction::SetMemorySize => {
                    match (self.stack.pop(), self.memory_names.contains(location)) {
                        (Some(SerializableWatType::I32), true) => {
                            Ok(self.stack.push(SerializableWatType::I32))
                        }
                        (Some(actual), true) => {
                            Err(WatError::type_error(&SerializableWatType::I32, &actual))
                        }
                        (Some(_), false) => Err(WatError::name_resolution_error(
                            location.clone(),
                            crate::NumLocationKind::Memory,
                        )),
                        (None, _) => Err(WatError::empty_stack(1)),
                    }
                }
            },
            SerializedInstruction::Memory {
                location,
                typ,
                is_storing,
                ..
            } => {
                if self.memory_names.contains(location) {
                    if *is_storing {
                        // Storing
                        if let Some(t) = self.stack.pop() {
                            typ.try_type_match(&t)?;
                            let idx = self
                                .stack
                                .pop()
                                .ok_or(WatError::not_enough_on_stack(2, 1))?;
                            SerializableWatType::I32.try_type_match(&idx)?;
                            Ok(())
                        } else {
                            Err(WatError::empty_stack(2))
                        }
                    } else {
                        // Loading
                        if let Some(t) = self.stack.pop() {
                            SerializableWatType::I32.try_type_match(&t)?;
                            Ok(self.stack.push(*typ))
                        } else {
                            Err(WatError::empty_stack(1))
                        }
                    }
                } else {
                    Err(WatError::name_resolution_error(
                        location.clone(),
                        crate::NumLocationKind::Memory,
                    ))
                }
            }
            SerializedInstruction::Const { typ, .. } => Ok(self.stack.push(*typ)),
            SerializedInstruction::Comparison { kind, typ } => match kind {
                crate::marker::ComparisonOperation::EqualZero => match self.stack.pop() {
                    Some(t) if &t == typ => Ok(self.stack.push(SerializableWatType::I32)),
                    Some(other) => Err(WatError::type_error(typ, &other)),
                    None => Err(WatError::empty_stack(1)),
                },
                _ => match (self.stack.pop(), self.stack.pop()) {
                    (Some(b), Some(a)) if &a == typ && &b == typ => {
                        Ok(self.stack.push(SerializableWatType::I32))
                    }
                    (Some(b), Some(a)) if &b == typ => Err(WatError::type_error(typ, &a)),
                    (Some(b), Some(_)) => Err(WatError::type_error(typ, &b)),
                    (Some(_), None) | (None, Some(_)) => Err(WatError::not_enough_on_stack(2, 1)),
                    (None, None) => Err(WatError::empty_stack(2)),
                },
            },
            SerializedInstruction::Arithmetic { typ, .. } => {
                match (self.stack.pop(), self.stack.pop()) {
                    (Some(b), Some(a)) if &a == typ && &b == typ => {
                        Ok(self.stack.push(SerializableWatType::I32))
                    }
                    (Some(b), Some(a)) if &b == typ => Err(WatError::type_error(typ, &a)),
                    (Some(b), Some(_)) => Err(WatError::type_error(typ, &b)),
                    (Some(_), None) | (None, Some(_)) => Err(WatError::not_enough_on_stack(2, 1)),
                    (None, None) => Err(WatError::empty_stack(2)),
                }
            }
            SerializedInstruction::Bitwise { kind, is_64_bit } => {
                let expected = if *is_64_bit {
                    &SerializableWatType::I64
                } else {
                    &SerializableWatType::I32
                };
                match kind {
                    crate::marker::BitwiseOperation::CountLeadingZero
                    | crate::marker::BitwiseOperation::CountTrailingZero
                    | crate::marker::BitwiseOperation::CountNonZero => {
                        match (self.stack.pop(), is_64_bit) {
                            (Some(SerializableWatType::I64), true) => {
                                Ok(self.stack.push(SerializableWatType::I64))
                            }
                            (Some(SerializableWatType::I32), false) => {
                                Ok(self.stack.push(SerializableWatType::I32))
                            }
                            (Some(t), _) => Err(WatError::type_error(expected, &t)),
                            (None, _) => Err(WatError::empty_stack(1)),
                        }
                    }
                    crate::marker::BitwiseOperation::And
                    | crate::marker::BitwiseOperation::Or
                    | crate::marker::BitwiseOperation::Xor
                    | crate::marker::BitwiseOperation::ShiftLeft
                    | crate::marker::BitwiseOperation::ShiftRightSigned
                    | crate::marker::BitwiseOperation::ShiftRightUnsigned
                    | crate::marker::BitwiseOperation::RotateLeft
                    | crate::marker::BitwiseOperation::RotateRight => {
                        match (self.stack.pop(), self.stack.pop()) {
                            (Some(b), Some(a)) if a == b => {
                                if &b == expected {
                                    Ok(self.stack.push(expected.clone()))
                                } else {
                                    Err(WatError::type_error(expected, &b))
                                }
                            }
                            (Some(SerializableWatType::I64), Some(a)) if *is_64_bit => {
                                Err(WatError::type_error(expected, &a))
                            }
                            (Some(SerializableWatType::I32), Some(a)) if !is_64_bit => {
                                Err(WatError::type_error(expected, &a))
                            }
                            (Some(b), Some(_)) => Err(WatError::type_error(expected, &b)),
                            (None, Some(_)) | (Some(_), None) => {
                                Err(WatError::not_enough_on_stack(2, 1))
                            }
                            (None, None) => Err(WatError::empty_stack(2)),
                        }
                    }
                }
            }
            SerializedInstruction::Float { kind, is_64_bit } => {
                let expected = if *is_64_bit {
                    &SerializableWatType::F64
                } else {
                    &SerializableWatType::F32
                };
                match kind {
                    crate::marker::FloatOperation::AbsoluteValue
                    | crate::marker::FloatOperation::Negation
                    | crate::marker::FloatOperation::Ceiling
                    | crate::marker::FloatOperation::Floor
                    | crate::marker::FloatOperation::Truncate
                    | crate::marker::FloatOperation::Nearest
                    | crate::marker::FloatOperation::SquareRoot => {
                        match (self.stack.pop(), is_64_bit) {
                            (Some(SerializableWatType::I64), true) => {
                                Ok(self.stack.push(SerializableWatType::F64))
                            }
                            (Some(SerializableWatType::I32), false) => {
                                Ok(self.stack.push(SerializableWatType::F32))
                            }
                            (Some(t), _) => Err(WatError::type_error(expected, &t)),
                            (None, _) => Err(WatError::empty_stack(1)),
                        }
                    }
                    crate::marker::FloatOperation::Minimum
                    | crate::marker::FloatOperation::Maximum
                    | crate::marker::FloatOperation::CopySign => {
                        match (self.stack.pop(), self.stack.pop()) {
                            (Some(b), Some(a)) if a == b => {
                                if &b == expected {
                                    Ok(self.stack.push(expected.clone()))
                                } else {
                                    Err(WatError::type_error(expected, &b))
                                }
                            }
                            (Some(SerializableWatType::F64), Some(a)) if *is_64_bit => {
                                Err(WatError::type_error(expected, &a))
                            }
                            (Some(SerializableWatType::F32), Some(a)) if !is_64_bit => {
                                Err(WatError::type_error(expected, &a))
                            }
                            (Some(b), Some(_)) => Err(WatError::type_error(expected, &b)),
                            (None, Some(_)) | (Some(_), None) => {
                                Err(WatError::not_enough_on_stack(2, 1))
                            }
                            (None, None) => Err(WatError::empty_stack(2)),
                        }
                    }
                }
            }
            SerializedInstruction::Cast(c) => {
                if let Some(t) = self.stack.pop() {
                    match c {
                        crate::marker::NumericConversionKind::WrapInt => {
                            SerializableWatType::I64.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::I32))
                        }
                        crate::marker::NumericConversionKind::SignedTruncF32ToI32
                        | crate::marker::NumericConversionKind::UnsignedTruncF32ToI32
                        | crate::marker::NumericConversionKind::Reinterpret32FToI => {
                            SerializableWatType::F32.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::I32))
                        }
                        crate::marker::NumericConversionKind::SignedTruncF64ToI32
                        | crate::marker::NumericConversionKind::UnsignedTruncF64ToI32 => {
                            SerializableWatType::F64.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::I32))
                        }
                        crate::marker::NumericConversionKind::SignedTruncF32ToI64
                        | crate::marker::NumericConversionKind::UnsignedTruncF32ToI64 => {
                            SerializableWatType::F32.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::I64))
                        }
                        crate::marker::NumericConversionKind::SignedTruncF64ToI64
                        | crate::marker::NumericConversionKind::UnsignedTruncF64ToI64
                        | crate::marker::NumericConversionKind::Reinterpret64FToI => {
                            SerializableWatType::F64.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::I64))
                        }
                        crate::marker::NumericConversionKind::SignedExtend
                        | crate::marker::NumericConversionKind::UnsignedExtend => {
                            SerializableWatType::I32.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::I64))
                        }
                        crate::marker::NumericConversionKind::SignedConvertI32ToF32
                        | crate::marker::NumericConversionKind::UnsignedConvertI32ToF32
                        | crate::marker::NumericConversionKind::Reinterpret32IToF => {
                            SerializableWatType::I32.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::F32))
                        }
                        crate::marker::NumericConversionKind::SignedConvertI64ToF32
                        | crate::marker::NumericConversionKind::UnsignedConvertI64ToF32 => {
                            SerializableWatType::I64.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::F32))
                        }
                        crate::marker::NumericConversionKind::SignedConvertI32ToF64
                        | crate::marker::NumericConversionKind::UnsignedConvertI32ToF64 => {
                            SerializableWatType::I32.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::F64))
                        }
                        crate::marker::NumericConversionKind::SignedConvertI64ToF64
                        | crate::marker::NumericConversionKind::UnsignedConvertI64ToF64
                        | crate::marker::NumericConversionKind::Reinterpret64IToF => {
                            SerializableWatType::I64.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::F64))
                        }
                        crate::marker::NumericConversionKind::DemoteFloat => {
                            SerializableWatType::F64.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::F32))
                        }
                        crate::marker::NumericConversionKind::PromoteFloat => {
                            SerializableWatType::F32.try_type_match(&t)?;
                            Ok(self.stack.push(SerializableWatType::F64))
                        }
                    }
                } else {
                    Err(WatError::empty_stack(1))
                }
            }
            SerializedInstruction::DefaultString(i) => {
                eprintln!("NOTE: This instruction is not type checked: {i}");
                Ok(())
            }
        }
    }

    pub fn current_stack(&self) -> &Vec<SerializableWatType> {
        &self.stack
    }
}
