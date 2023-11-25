//! Validator module for custom WAT AST.
//!
//! Based loosely on algorithm described in <https://webassembly.github.io/spec/core/appendix/algorithm.html>

use std::collections::{HashMap, HashSet};

use crate::{
    error::{WatError, WatResult},
    instruction::SerializedInstruction,
    marker::{self, SerializableWatType, SimpleInstruction},
    InterpreterStructure,
};

/// Try to convert a name to a [usize] index,
/// returning Ok(index) | Err(orignal name)
pub fn try_name_to_index(name: &str) -> Result<usize, &str> {
    str::parse::<usize>(&name).map_err(|_err| name)
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueMapping<Value> {
    values: Vec<Value>,
    mapping: HashMap<String, usize>,
}

impl<Value> ValueMapping<Value> {
    pub fn new() -> Self {
        Self {
            values: Vec::new(),
            mapping: HashMap::new(),
        }
    }

    /// Get a possible reference to value by its index
    pub fn get_by_index(&self, index: usize) -> Option<&Value> {
        self.values.get(index)
    }

    /// Get a possible mutable reference to value by its index
    pub fn get_mut_by_index(&mut self, index: usize) -> Option<&mut Value> {
        self.values.get_mut(index)
    }

    /// Get a possible reference to value by its name
    pub fn get_by_name(&self, name: &str) -> Option<&Value> {
        self.mapping
            .get(name)
            .and_then(|index| self.get_by_index(*index))
    }

    /// Get a possible mutable reference to value by its name
    pub fn get_mut_by_name(&mut self, name: &str) -> Option<&mut Value> {
        let Some(index) = self.mapping.get(name) else {
            return None;
        };
        self.get_mut_by_index(*index)
    }

    /// Get a possoble reference to value
    ///
    /// Will try to convert the key to an index,
    /// if success, then get by that index,
    /// otherwise, get by name
    pub fn get(&self, key: &str) -> Option<&Value> {
        str::parse::<usize>(key).map_or_else(
            |_err| self.get_by_name(key),
            |index| self.get_by_index(index),
        )
    }

    /// Get a possoble reference to value
    ///
    /// Will try to convert the key to an index,
    /// if success, then get by that index,
    /// otherwise, get by name
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match str::parse::<usize>(key) {
            Ok(index) => self.get_mut_by_index(index),
            Err(_) => self.get_mut_by_name(key),
        }
    }
}

impl<Value> FromIterator<(Option<String>, Value)> for ValueMapping<Value> {
    fn from_iter<T: IntoIterator<Item = (Option<String>, Value)>>(iter: T) -> Self {
        iter.into_iter()
            .enumerate()
            .fold(Self::new(), |mut this, (idx, (maybe_name, val))| {
                if let Some(name) = maybe_name {
                    this.mapping.insert(name, idx);
                }
                this.values.push(val);
                this
            })
    }
}

impl<Value> FromIterator<(String, Value)> for ValueMapping<Value> {
    fn from_iter<T: IntoIterator<Item = (String, Value)>>(iter: T) -> Self {
        iter.into_iter()
            .map(|(name, val)| (Some(name), val))
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFrame {
    opcode: marker::BlockKind,
    label: Option<String>,
    start_types: Vec<SerializableWatType>,
    end_types: Vec<SerializableWatType>,
    height: usize,
    unreachable: bool,
}

impl ControlFrame {
    pub fn is_if(&self) -> bool {
        matches!(self.opcode, marker::BlockKind::If)
    }
}

/// A simple Wat validator, checking both stack is correctly sized and has correct type at each instruction
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validator {
    /// Stack of types being processed
    value_stack: Vec<SerializableWatType>,
    control_stack: Vec<ControlFrame>,
    /// Global values mapping name to (mutablitiy, type)
    globals: ValueMapping<(bool, SerializableWatType)>,
    memory_names: HashSet<String>,
    functions: ValueMapping<(Vec<SerializableWatType>, Vec<SerializableWatType>)>,
}

impl Validator {
    pub fn new(structure: &InterpreterStructure) -> Self {
        Validator {
            value_stack: Vec::new(),
            control_stack: Vec::new(),
            globals: structure
                .globals
                .iter()
                .map(|g| (g.name.clone(), (g.is_mutable, g.typ)))
                .collect(),
            memory_names: structure.memory.iter().map(|m| m.name.clone()).collect(),
            functions: structure
                .func
                .iter()
                .map(|f| {
                    (
                        f.name(),
                        (
                            f.info.input.iter().map(|(_, typ)| *typ).collect(),
                            f.info.output.clone(),
                        ),
                    )
                })
                .collect(),
        }
    }

    /// Reset both value and control stacks
    fn reset_stack(&mut self) {
        self.value_stack.clear();
        self.control_stack.clear();
    }

    /// Push type onto the value stack
    fn push_val(&mut self, typ: SerializableWatType) {
        self.value_stack.push(typ);
    }

    /// Pop value from value stack, return value or empty stack error
    fn pop_val(&mut self) -> WatResult<SerializableWatType> {
        self.value_stack.pop().ok_or(WatError::empty_stack(1))
    }

    /// Pop the expected type from the value stack, returning value or error
    fn expected_pop_val(
        &mut self,
        expected: &SerializableWatType,
    ) -> WatResult<SerializableWatType> {
        let actual = self.pop_val()?;
        if &actual != expected {
            Err(WatError::unexpected_type(expected, &actual))
        } else {
            Ok(actual)
        }
    }

    /// Do multiple push operations on value stack
    fn push_vals(&mut self, types: &[SerializableWatType]) {
        types.iter().for_each(|typ| {
            self.push_val(*typ);
        });
    }

    /// Do multiple pop operations on value stack
    fn pop_vals(&mut self, types: &[SerializableWatType]) -> WatResult<Vec<SerializableWatType>> {
        types
            .iter()
            .rev()
            .map(|typ| self.expected_pop_val(typ))
            // Second rev to do prepending
            .rev()
            .collect::<WatResult<Vec<_>>>()
    }

    fn push_control(
        &mut self,
        opcode: marker::BlockKind,
        label: &String,
        input: Vec<SerializableWatType>,
        output: Vec<SerializableWatType>,
    ) {
        let frame = ControlFrame {
            opcode: opcode,
            label: if label.is_empty() {
                None
            } else {
                Some(label.clone())
            },
            start_types: input,
            end_types: output,
            height: self.value_stack.len(),
            unreachable: false,
        };
        self.control_stack.push(frame);
    }

    fn pop_control(&mut self) -> WatResult<ControlFrame> {
        let Some(frame) = self.control_stack.pop() else {
            return Err(WatError::empty_stack(1));
        };
        self.pop_vals(&frame.end_types)?;
        if self.value_stack.len() != frame.height {
            Err(WatError::not_enough_on_stack(
                frame.height,
                self.value_stack.len(),
            ))
        } else {
            Ok(frame)
        }
    }

    fn label_types(&self, frame: ControlFrame) -> Vec<SerializableWatType> {
        if matches!(frame.opcode, marker::BlockKind::Loop) {
            frame.start_types
        } else {
            frame.end_types
        }
    }

    fn unreachable(&mut self) {
        if let Some(top_control) = self.control_stack.last_mut() {
            self.value_stack.reserve(top_control.height);
            top_control.unreachable = true;
        }
    }

    fn validate(
        &mut self,
        instruction: &SerializedInstruction,
        output: &[SerializableWatType],
        locals: &ValueMapping<SerializableWatType>,
    ) -> WatResult<()> {
        match instruction {
            SerializedInstruction::Simple(s) => match s {
                SimpleInstruction::Unreachable => Ok(self.unreachable()),
                SimpleInstruction::Nop => Ok(()),
                SimpleInstruction::Drop => {
                    self.pop_val()?;
                    Ok(())
                }
                SimpleInstruction::Return => {
                    self.pop_vals(output)?;
                    Ok(())
                }
            },
            SerializedInstruction::Block { label, kind, inout } => match kind {
                marker::BlockKind::Block | marker::BlockKind::Loop => {
                    // SAFETY: Block is always gaurenteed to have an input-output section
                    let input = &inout.as_ref().unwrap().get_input_types();
                    let output = &inout.as_ref().unwrap().output;
                    self.pop_vals(&input)?;
                    Ok(self.push_control(*kind, label, input.to_vec(), output.to_vec()))
                }
                marker::BlockKind::If => {
                    // SAFETY: Block is always gaurenteed to have an input-output section
                    let input = &inout.as_ref().unwrap().get_input_types();
                    let output = &inout.as_ref().unwrap().output;
                    self.expected_pop_val(&SerializableWatType::I32)?;
                    self.pop_vals(&input)?;
                    Ok(self.push_control(*kind, label, input.to_vec(), output.to_vec()))
                }
                marker::BlockKind::Else => {
                    let frame = self.pop_control()?;
                    if !frame.is_if() {
                        return Err(WatError::else_without_if_error());
                    }
                    Ok(self.push_control(*kind, label, frame.start_types, frame.end_types))
                }
                marker::BlockKind::End => {
                    let frame = self.pop_control()?;
                    Ok(self.push_vals(&frame.end_types))
                }
            },
            SerializedInstruction::Branch {
                default_label,
                other_labels,
                is_conditional,
            } => {
                let default_frame = self.try_get_control_frame(default_label)?;
                let default_vals = self.label_types(default_frame);
                if other_labels.is_empty() {
                    if *is_conditional {
                        self.expected_pop_val(&SerializableWatType::I32)?;
                        self.pop_vals(&default_vals)?;
                        Ok(self.push_vals(&default_vals))
                    } else {
                        self.pop_vals(&default_vals)?;
                        Ok(self.unreachable())
                    }
                } else {
                    self.expected_pop_val(&SerializableWatType::I32)?;
                    let arity = default_vals.len();
                    other_labels.iter().try_for_each(|label| {
                        let frame = self.try_get_control_frame(label)?;
                        let vals = self.label_types(frame);
                        if vals.len() != arity {
                            return Err(WatError::mismatched_inout(&default_vals, &vals, false));
                        }
                        let popped = &self.pop_vals(&vals)?;
                        Ok(self.push_vals(&popped))
                    })?;
                    self.pop_vals(&default_vals)?;
                    Ok(self.unreachable())
                }
            }
            SerializedInstruction::Call { index, inout } => {
                if self.functions.get(&index).is_some() {
                    // Assumes success on the called function
                    self.pop_vals(&inout.get_input_types())?;
                    Ok(self.push_vals(&inout.output))
                } else {
                    Err(WatError::name_resolution_error(
                        &index,
                        crate::NumLocationKind::Function,
                    ))
                }
            }
            SerializedInstruction::Data { kind, location } => match kind {
                marker::DataInstruction::GetLocal => {
                    if let Some(typ) = locals.get(&location) {
                        Ok(self.push_val(*typ))
                    } else {
                        Err(WatError::local_resolution_error(location))
                    }
                }
                marker::DataInstruction::GetGlobal => {
                    if let Some((_, typ)) = self.globals.get(&location) {
                        Ok(self.push_val(*typ))
                    } else {
                        Err(WatError::name_resolution_error(
                            location,
                            crate::NumLocationKind::Global,
                        ))
                    }
                }
                marker::DataInstruction::SetLocal => {
                    if let Some(typ) = locals.get(&location) {
                        self.expected_pop_val(typ)?;
                        Ok(())
                    } else {
                        Err(WatError::local_resolution_error(location))
                    }
                }
                marker::DataInstruction::SetGlobal => {
                    let Some((mutable, typ)) = self.globals.get(&location).cloned() else {
                        return Err(WatError::name_resolution_error(
                            location,
                            crate::NumLocationKind::Global,
                        ));
                    };
                    if mutable {
                        self.expected_pop_val(&typ)?;
                        Ok(())
                    } else {
                        Err(WatError::setting_immutable_global_error(&location))
                    }
                }
                marker::DataInstruction::TeeLocal => {
                    if let Some(typ) = locals.get(&location) {
                        self.expected_pop_val(typ)?;
                        Ok(self.push_val(*typ))
                    } else {
                        Err(WatError::local_resolution_error(location))
                    }
                }
                marker::DataInstruction::GetMemorySize => {
                    if self.memory_names.contains(location) {
                        Ok(self.push_val(SerializableWatType::I32))
                    } else {
                        Err(WatError::name_resolution_error(
                            location,
                            crate::NumLocationKind::Memory,
                        ))
                    }
                }

                marker::DataInstruction::SetMemorySize => {
                    if self.memory_names.contains(location) {
                        self.expected_pop_val(&SerializableWatType::I32)?;
                        Ok(self.push_val(SerializableWatType::I32))
                    } else {
                        Err(WatError::name_resolution_error(
                            location,
                            crate::NumLocationKind::Memory,
                        ))
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
                        self.expected_pop_val(&typ)?;
                        self.expected_pop_val(&SerializableWatType::I32)?;
                        Ok(())
                    } else {
                        self.expected_pop_val(&SerializableWatType::I32)?;
                        Ok(self.push_val(*typ))
                    }
                } else {
                    Err(WatError::name_resolution_error(
                        location,
                        crate::NumLocationKind::Memory,
                    ))
                }
            }
            SerializedInstruction::Const { typ, .. } => Ok(self.push_val(*typ)),
            SerializedInstruction::Arithmetic { typ, kind } => {
                self.expected_pop_val(typ)?;
                self.expected_pop_val(typ)?;
                Ok(self.push_val(*typ))
            }
            SerializedInstruction::Comparison { typ, kind } => {
                if matches!(kind, crate::marker::ComparisonOperation::EqualZero) {
                    Ok(self.push_val(SerializableWatType::I32))
                } else {
                    self.expected_pop_val(typ)?;
                    self.expected_pop_val(typ)?;
                    Ok(self.push_val(SerializableWatType::I32))
                }
            }
            SerializedInstruction::Bitwise { kind, is_64_bit } => {
                let typ = if *is_64_bit {
                    SerializableWatType::I64
                } else {
                    SerializableWatType::I32
                };
                if matches!(
                    kind,
                    crate::marker::BitwiseOperation::CountLeadingZero
                        | crate::marker::BitwiseOperation::CountTrailingZero
                        | crate::marker::BitwiseOperation::CountNonZero
                ) {
                    self.expected_pop_val(&typ)?;
                    Ok(self.push_val(typ))
                } else {
                    self.expected_pop_val(&typ)?;
                    self.expected_pop_val(&typ)?;
                    Ok(self.push_val(typ))
                }
            }
            SerializedInstruction::Float { kind, is_64_bit } => {
                let typ = if *is_64_bit {
                    SerializableWatType::F64
                } else {
                    SerializableWatType::F32
                };
                if matches!(
                    kind,
                    crate::marker::FloatOperation::Minimum
                        | crate::marker::FloatOperation::Maximum
                        | crate::marker::FloatOperation::CopySign
                ) {
                    self.expected_pop_val(&typ)?;
                    self.expected_pop_val(&typ)?;
                    Ok(self.push_val(typ))
                } else {
                    self.expected_pop_val(&typ)?;
                    Ok(self.push_val(typ))
                }
            }
            SerializedInstruction::Conversion(c) => match c {
                marker::NumericConversionKind::WrapInt => {
                    self.expected_pop_val(&SerializableWatType::I64)?;
                    Ok(self.push_val(SerializableWatType::I32))
                }
                marker::NumericConversionKind::SignedTruncF32ToI32
                | marker::NumericConversionKind::UnsignedTruncF32ToI32 => {
                    self.expected_pop_val(&SerializableWatType::F32)?;
                    Ok(self.push_val(SerializableWatType::I32))
                }
                marker::NumericConversionKind::SignedTruncF64ToI32
                | marker::NumericConversionKind::UnsignedTruncF64ToI32 => {
                    self.expected_pop_val(&SerializableWatType::F64)?;
                    Ok(self.push_val(SerializableWatType::I32))
                }
                marker::NumericConversionKind::SignedTruncF32ToI64
                | marker::NumericConversionKind::UnsignedTruncF32ToI64 => {
                    self.expected_pop_val(&SerializableWatType::F32)?;
                    Ok(self.push_val(SerializableWatType::I64))
                }
                marker::NumericConversionKind::SignedTruncF64ToI64
                | marker::NumericConversionKind::UnsignedTruncF64ToI64 => {
                    self.expected_pop_val(&SerializableWatType::F64)?;
                    Ok(self.push_val(SerializableWatType::I64))
                }
                marker::NumericConversionKind::SignedExtend
                | marker::NumericConversionKind::UnsignedExtend => {
                    self.expected_pop_val(&SerializableWatType::I32)?;
                    Ok(self.push_val(SerializableWatType::I64))
                }
                marker::NumericConversionKind::SignedConvertI32ToF32
                | marker::NumericConversionKind::UnsignedConvertI32ToF32 => {
                    self.expected_pop_val(&SerializableWatType::I32)?;
                    Ok(self.push_val(SerializableWatType::F32))
                }
                marker::NumericConversionKind::SignedConvertI64ToF32
                | marker::NumericConversionKind::UnsignedConvertI64ToF32 => {
                    self.expected_pop_val(&SerializableWatType::I64)?;
                    Ok(self.push_val(SerializableWatType::F32))
                }
                marker::NumericConversionKind::SignedConvertI32ToF64
                | marker::NumericConversionKind::UnsignedConvertI32ToF64 => {
                    self.expected_pop_val(&SerializableWatType::I32)?;
                    Ok(self.push_val(SerializableWatType::F64))
                }
                marker::NumericConversionKind::SignedConvertI64ToF64
                | marker::NumericConversionKind::UnsignedConvertI64ToF64 => {
                    self.expected_pop_val(&SerializableWatType::I64)?;
                    Ok(self.push_val(SerializableWatType::F64))
                }
                marker::NumericConversionKind::DemoteFloat => {
                    self.expected_pop_val(&SerializableWatType::F64)?;
                    Ok(self.push_val(SerializableWatType::F32))
                }
                marker::NumericConversionKind::PromoteFloat => {
                    self.expected_pop_val(&SerializableWatType::F32)?;
                    Ok(self.push_val(SerializableWatType::F64))
                }
                marker::NumericConversionKind::Reinterpret32FToI => {
                    self.expected_pop_val(&SerializableWatType::F32)?;
                    Ok(self.push_val(SerializableWatType::I32))
                }
                marker::NumericConversionKind::Reinterpret32IToF => {
                    self.expected_pop_val(&SerializableWatType::I32)?;
                    Ok(self.push_val(SerializableWatType::F32))
                }
                marker::NumericConversionKind::Reinterpret64FToI => {
                    self.expected_pop_val(&SerializableWatType::F64)?;
                    Ok(self.push_val(SerializableWatType::I64))
                }
                marker::NumericConversionKind::Reinterpret64IToF => {
                    self.expected_pop_val(&SerializableWatType::I64)?;
                    Ok(self.push_val(SerializableWatType::F64))
                }
            },
            SerializedInstruction::DefaultString(msg) => Err(WatError::unimplemented_error(
                &format!("Instruction not supported: {msg}"),
            )),
        }
    }

    fn try_get_control_frame(&self, label: &str) -> WatResult<ControlFrame> {
        Ok(match try_name_to_index(label) {
            Ok(index) => {
                self.control_stack
                    .get(index)
                    .ok_or(WatError::index_out_of_range_range(
                        self.control_stack.len(),
                        index,
                    ))?
            }
            Err(name) => self
                .control_stack
                .iter()
                .find_map(|cf| {
                    cf.label
                        .as_ref()
                        .map(|label| name == label)
                        .and_then(|is_same| is_same.then_some(cf))
                })
                .ok_or(WatError::label_resolution_error(name))?,
        }
        .clone())
    }

    pub fn validate_function(
        &mut self,
        instuctions: &[SerializedInstruction],
        params: &[(Option<String>, SerializableWatType)],
        locals: &[(Option<String>, SerializableWatType)],
        results: &[SerializableWatType],
    ) -> WatResult<()> {
        self.reset_stack();
        let local_vars = params.iter().chain(locals.iter()).cloned().collect();
        for instruction in instuctions {
            self.validate(instruction, results, &local_vars)?;
        }
        self.pop_vals(results)?;
        if !self.value_stack.is_empty() {
            Err(WatError::extra_items_on_stack_error(&self.value_stack))
        } else {
            Ok(())
        }
    }
}
