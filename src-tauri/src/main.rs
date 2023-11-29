// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use helper::SerializedNumber;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::collections::HashMap;
use wast::{
    self,
    core::{DataVal, Expression, Func, Local, ModuleField},
    parser::{self, ParseBuffer},
    token::Id,
    Wat,
};

mod error;
mod helper;
mod instruction;
mod marker;
mod validator;

use error::{WatError, WatResult};
use instruction::{index_to_string, InputOutput, SerializedInstruction, SerializedInstructionTree};
use validator::Validator;

use marker::SerializableWatType;

/// A basic Wa(s)t Function
///
/// ## Note:
/// Does not work with imported functions, as it assumes nothing about other modules
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct WastFunc {
    info: instruction::InputOutput,
    locals: Vec<(Option<String>, SerializableWatType)>,
    block: SerializedInstructionTree,
}

impl WastFunc {
    pub fn try_new(
        info: instruction::InputOutput,
        locals: &[Local],
        expression: &Expression,
    ) -> WatResult<Self> {
        let locals = locals
            .iter()
            .map(|l| match SerializableWatType::try_from(l.ty) {
                Ok(ty) => Ok((l.id.map(|i| i.name().to_string()), ty)),
                Err(err) => Err(err),
            })
            .collect::<Result<Vec<_>, error::WatError>>()?;
        let func_name = info.index.clone().unwrap_or_default();
        Ok(WastFunc {
            info,
            locals,
            block: SerializedInstructionTree::try_from_instruction(&func_name, &expression.instrs)?,
        })
    }

    pub fn set_name_from_number(&mut self, index: usize) {
        self.info.index = Some(index.to_string());
    }

    pub fn name(&self) -> Option<String> {
        self.info.index.clone()
    }
}

impl TryFrom<&Func<'_>> for WastFunc {
    type Error = error::WatError;

    fn try_from(value: &Func<'_>) -> Result<Self, Self::Error> {
        // dbg!(value.id);
        // dbg!(value.ty.index);
        let mut info = InputOutput::try_from(&value.ty)?;
        if let Some(id) = value.id {
            info.set_name_if_none(id.name());
        }

        match &value.kind {
            wast::core::FuncKind::Import(_) => Err(error::WatError::unimplemented_error(
                "Import functions are not supported yet.",
            )),
            wast::core::FuncKind::Inline { locals, expression } => {
                WastFunc::try_new(info, locals, expression)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Type, derive_more::Display)]
pub enum NumLocationKind {
    Function,
    Global,
    Memory,
    Type,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GlobalData {
    name: String,
    typ: SerializableWatType,
    is_mutable: bool,
    val: SerializedNumber,
}

pub fn const_eval_expr(
    instrs: &[SerializedInstruction],
    expected_type: Option<SerializableWatType>,
) -> WatResult<SerializedNumber> {
    match instrs[..] {
        [SerializedInstruction::Const { typ, value }]
            if expected_type.is_some_and(|t| typ == t) =>
        {
            Ok(value.clone())
        }
        [SerializedInstruction::Const { typ, value }] => Ok(value.clone()),
        [] => Err(WatError::no_instruction_provided("Const")),
        _ => Err(WatError::non_initializer_expression()),
    }
}

impl GlobalData {
    pub fn try_new(
        name: String,
        gtyp: SerializableWatType,
        is_mutable: bool,
        instructions: Vec<SerializedInstruction>,
    ) -> WatResult<Self> {
        Ok(Self {
            name,
            typ: gtyp,
            is_mutable,
            val: const_eval_expr(&instructions, Some(gtyp))?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct DataValue {
    id: String,
    is_string: bool,
    data: Vec<u8>,
}

impl From<DataVal<'_>> for DataValue {
    fn from(value: DataVal) -> Self {
        match value {
            DataVal::String(s) => Self {
                id: String::default(),
                is_string: true,
                data: s.to_vec(),
            },
            DataVal::Integral(i) => Self {
                id: String::default(),
                is_string: false,
                data: i,
            },
        }
    }
}

impl DataValue {
    pub fn clone_from(value: &DataVal) -> Self {
        match value {
            DataVal::String(s) => Self {
                id: String::default(),
                is_string: true,
                data: s.to_vec(),
            },
            DataVal::Integral(i) => Self {
                id: String::default(),
                is_string: false,
                data: i.to_vec(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct MemoryData {
    name: String,
    min: SerializedNumber,
    max: SerializedNumber,
    is_32: bool,
    is_shared: bool,
    data: HashMap<u32, DataValue>,
}

impl MemoryData {
    pub fn new(
        name: String,
        min: i64,
        max: Option<i64>,
        is_32: bool,
        is_shared: bool,
        data: HashMap<u32, DataValue>,
    ) -> Self {
        let min = min.into();
        let max = max.into();
        Self {
            name,
            min,
            max,
            is_32,
            is_shared,
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct InterpreterStructure {
    pub(crate) name: String,
    pub(crate) exported: HashMap<String, (NumLocationKind, u32)>,
    pub(crate) globals: Vec<GlobalData>,
    pub(crate) memory: Vec<MemoryData>,
    // Data not currently bound to a specific memory location
    pub(crate) free_data: Vec<DataValue>,
    pub(crate) func: Vec<WastFunc>,
    /// Optional start function for initalization
    pub(crate) start: Option<String>,
}

impl InterpreterStructure {
    const PAGE_SIZE_AS_BYTES: u32 = 65536;

    /// Try to create a new interpreter structure
    pub fn try_new(_text: &str, fields: &[ModuleField], name: &Option<Id>) -> WatResult<Self> {
        let mut exported: HashMap<String, (NumLocationKind, u32)> = HashMap::new();
        let mut globals: Vec<GlobalData> = Vec::new();
        let mut memory: Vec<MemoryData> = Vec::new();
        let mut free_data: Vec<DataValue> = Vec::new();
        let mut func: Vec<WastFunc> = Vec::new();
        let mut start = None;
        // let mut passive_data = Vec::new();
        // let mut active_data = Vec::new();
        // let mut start = 0;
        for (_i, field) in fields.iter().enumerate() {
            match field {
                ModuleField::Import(_) => unimplemented!("Import field not implemented"),
                ModuleField::Export(e) => match e.kind {
                    wast::core::ExportKind::Func => {
                        for (i, f) in func.iter().enumerate() {
                            if f.name()
                                .as_ref()
                                .is_some_and(|item| item == &instruction::index_to_string(&e.item))
                            {
                                exported
                                    .insert(
                                        e.name.to_string(),
                                        (NumLocationKind::Function, i as u32),
                                    )
                                    .map_or(Ok(()), |_| {
                                        Err(WatError::duplicate_name_error(e.name))
                                    })?;
                                break;
                            }
                        }
                    }
                    wast::core::ExportKind::Memory => {
                        for (i, m) in memory.iter().enumerate() {
                            if m.name == instruction::index_to_string(&e.item) {
                                exported
                                    .insert(e.name.to_string(), (NumLocationKind::Memory, i as u32))
                                    .map_or(Ok(()), |_| {
                                        Err(WatError::duplicate_name_error(&m.name))
                                    })?;
                                break;
                            }
                        }
                    }
                    wast::core::ExportKind::Global => {
                        for (i, g) in globals.iter().enumerate() {
                            if g.name == instruction::index_to_string(&e.item) {
                                exported
                                    .insert(e.name.to_string(), (NumLocationKind::Global, i as u32))
                                    .map_or(Ok(()), |_| {
                                        Err(WatError::duplicate_name_error(e.name))
                                    })?;
                                break;
                            }
                        }
                    }
                    wast::core::ExportKind::Table => todo!("Export Tables not implemented"),
                    wast::core::ExportKind::Tag => todo!("Export Tags not implemented"),
                },
                ModuleField::Global(g) => {
                    for name in &g.exports.names {
                        exported
                            .insert(
                                name.to_string(),
                                (NumLocationKind::Global, globals.len() as u32),
                            )
                            .map_or(Ok(()), |_| Err(WatError::duplicate_name_error(name)))?;
                    }
                    match &g.kind {
                        wast::core::GlobalKind::Import(_) => {
                            Err(error::WatError::unimplemented_error(
                                "Imported globals not yet implemented.",
                            ))?
                        }
                        wast::core::GlobalKind::Inline(e) => {
                            globals.push(GlobalData::try_new(
                                g.id.map(|id| id.name().to_string()).unwrap_or_default(),
                                g.ty.ty.try_into()?,
                                g.ty.mutable,
                                e.instrs
                                    .iter()
                                    .map(|ins| ins.try_into())
                                    .collect::<Result<_, _>>()?,
                            )?);
                        }
                    }
                }
                ModuleField::Func(f) => {
                    for name in &f.exports.names {
                        exported
                            .insert(
                                name.to_string(),
                                (NumLocationKind::Function, func.len() as u32),
                            )
                            .map_or(Ok(()), |_| Err(WatError::duplicate_name_error(name)))?;
                    }
                    let mut function = WastFunc::try_from(f)?;
                    if function.name().is_none() {
                        function.set_name_from_number(func.len())
                    };
                    func.push(function);
                }
                ModuleField::Start(s) => {
                    // Parsing gaurentees only one start
                    start = Some(index_to_string(s));
                }
                ModuleField::Memory(m) => {
                    let mem_name = m.id.map(|id| id.name().to_string()).unwrap_or_default();
                    for name in &m.exports.names {
                        exported
                            .insert(
                                name.to_string(),
                                (NumLocationKind::Memory, memory.len() as u32),
                            )
                            .map_or(Ok(()), |_| Err(WatError::duplicate_name_error(name)))?;
                    }
                    match &m.kind {
                        wast::core::MemoryKind::Import { import: _, ty: _ } => {
                            Err(error::WatError::unimplemented_error(
                                "Imported memory not yet implemented.",
                            ))?
                        }
                        wast::core::MemoryKind::Normal(mt) => match mt {
                            wast::core::MemoryType::B32 { limits, shared } => {
                                memory.push(MemoryData::new(
                                    mem_name,
                                    limits.min as i64,
                                    limits.max.map(|n| n as i64),
                                    true,
                                    *shared,
                                    HashMap::new(),
                                ));
                            }
                            wast::core::MemoryType::B64 { limits, shared } => {
                                memory.push(MemoryData::new(
                                    mem_name,
                                    limits.min as i64,
                                    limits.max.map(|i| i as i64),
                                    false,
                                    *shared,
                                    HashMap::new(),
                                ));
                            }
                        },
                        wast::core::MemoryKind::Inline { is_32, data } => {
                            // Size calculated from data
                            let (final_offset, data) = data
                                .iter()
                                .map(|val| DataValue::clone_from(val))
                                .fold((0 as u32, HashMap::new()), |(offset, mut map), val| {
                                    let next_offset = offset + val.data.len() as u32;
                                    map.insert(offset, val);
                                    // Next offset = prev offset + len of curr val
                                    (next_offset, map)
                                });
                            // Mem size gives exact size by page size, rounded up
                            let mem_size = final_offset / Self::PAGE_SIZE_AS_BYTES
                                + if final_offset % Self::PAGE_SIZE_AS_BYTES != 0 {
                                    1
                                } else {
                                    0
                                };
                            memory.push(MemoryData::new(
                                mem_name,
                                0,
                                Some(mem_size as i64),
                                *is_32,
                                false,
                                data,
                            ));
                        }
                    }
                }
                ModuleField::Data(d) => {
                    // d.data
                    // d.id
                    let id = d.id.map_or(String::default(), |id| id.name().to_string());
                    let data = d
                        .data
                        .iter()
                        .flat_map(|val| match val {
                            DataVal::String(s) => s.to_vec(),
                            DataVal::Integral(i) => i.to_vec(),
                        })
                        .collect();
                    match &d.kind {
                        // Passive = exist but not yet loaded to memory -> put in free data
                        wast::core::DataKind::Passive => free_data.push(DataValue {
                            id,
                            is_string: d.data.iter().all(|v| matches!(v, DataVal::String(_))),
                            data,
                        }),
                        // Active = load directly to memory -> put in memory (or wait until memory is initalized)
                        wast::core::DataKind::Active {
                            memory: idx,
                            offset,
                        } => {
                            let mem_name = index_to_string(idx);
                            // Memory already defined
                            if let Some(mem) = memory
                                .iter_mut()
                                .find(|m| !m.name.is_empty() && m.name == mem_name)
                            {
                                let expr = offset
                                    .instrs
                                    .iter()
                                    .map(|inst| inst.try_into())
                                    .collect::<Result<Vec<_>, _>>()?;
                                mem.data.insert(
                                    const_eval_expr(&expr, None)?.try_into()?,
                                    DataValue {
                                        id,
                                        is_string: d
                                            .data
                                            .iter()
                                            .all(|dv| matches!(dv, DataVal::String(_))),
                                        data,
                                    },
                                );
                            } else {
                                // TODO: Possibly add a temporary hashmap that will store data before memory creation
                                Err(WatError::unimplemented_error(&format!("Currently cannot add data to memory not already defined: memory {} is not defined at this point", mem_name)))?
                            }
                        }
                    }
                }
                ModuleField::Type(_) => todo!("Type field not implemented"),
                ModuleField::Rec(_) => todo!("Rec field not implemented"),
                ModuleField::Table(_) => todo!("Table field not implemented"),
                ModuleField::Elem(_) => todo!("Element field not implemented"),
                ModuleField::Tag(_) => todo!("Tag field not implemented"),
                ModuleField::Custom(_) => todo!("Custom field not implemented"),
            }
        }
        let interp_struct = InterpreterStructure {
            name: name.map(|id| id.name().to_string()).unwrap_or_default(),
            exported,
            globals,
            memory,
            free_data,
            func,
            start,
        };
        interp_struct.validate()?;
        Ok(interp_struct)
    }

    /// Validate that the structure is correct, check all types match, and stack flow is correct.
    pub fn validate(&self) -> WatResult<()> {
        let mut validator = Validator::new(self);
        for func in &self.func {
            validator.validate_function(
                &func.block.array,
                &func.info.input,
                &func.locals,
                &func.info.output,
            )?;
        }
        Ok(())

        // // TODO: Remove the need for .clone()
        // // Functions with parameter and result types
        // let funcs: HashMap<_, _> = self
        //     .func
        //     .iter()
        //     .enumerate()
        //     .flat_map(|(i, f)| {
        //         let params: Vec<_> = f.info.input.iter().map(|(_, t)| *t).collect();
        //         let results = &f.info.output;
        //         if let Some(name) = f.name() {
        //             [
        //                 (i.to_string(), (params.clone(), results.clone())),
        //                 (name, (params, results.clone())),
        //             ]
        //         } else {
        //             [
        //                 (i.to_string(), (params.clone(), results.clone())),
        //                 (i.to_string(), (params, results.clone())),
        //             ]
        //         }
        //     })
        //     .collect();
        // for func in &self.func {
        //     let mut validator = Validator::new(
        //         self.globals
        //             .iter()
        //             .enumerate()
        //             .flat_map(|(i, g)| {
        //                 [
        //                     (i.to_string(), (g.is_mutable, g.typ)),
        //                     (g.name.clone(), (g.is_mutable, g.typ)),
        //                 ]
        //             })
        //             .collect(),
        //         func.info
        //             .input
        //             .iter()
        //             .chain(func.locals.iter())
        //             .enumerate()
        //             .flat_map(|(i, l)| {
        //                 if let Some(name) = l.0.clone() {
        //                     [(i.to_string(), l.1), (name, l.1)]
        //                 } else {
        //                     [(i.to_string(), l.1), (i.to_string(), l.1)]
        //                 }
        //             })
        //             .collect(),
        //         funcs.clone(),
        //         self.memory.iter().map(|m| m.name.clone()).collect(),
        //         func.info.output.clone(),
        //     );
        //     dbg!(&func.block);
        //     // for instruction in func.block.get_root() {
        //     //     validator.process(instruction)?;
        //     // }
        // }
        // Ok(())
    }
}

/// Primary transformation function
#[tauri::command]
#[specta::specta]
fn inner_transform(text: &str) -> error::WatResult<InterpreterStructure> {
    // Note: New only builds the buffer and is currently infallible
    let buffer = ParseBuffer::new(text).map_err(WatError::parsing_error)?;
    // Combined lexing and parsing step
    let mut module = match parser::parse::<Wat>(&buffer).map_err(WatError::parsing_error)? {
        Wat::Module(m) => m,
        Wat::Component(_) => {
            return Err(error::WatError::unimplemented_error(
                "Cannot compile components currently.",
            ));
        }
    };
    // dbg!(&module);
    let final_result = match module.kind {
        wast::core::ModuleKind::Text(ref fields) => {
            InterpreterStructure::try_new(text, fields, &module.id)
        }
        wast::core::ModuleKind::Binary(_) => Err(error::WatError::unimplemented_error(
            "Unimplemented Error: Cannot binary type currently.",
        )),
    };
    // Resolve and immediately throw-away
    let _ = module.resolve().map_err(WatError::resolution_error)?;
    // Print for debug purposes, it does change module, so need to resolve name separately.
    // println!("{}", wasmprinter::print_bytes(module.encode().unwrap()).unwrap());
    final_result
}

/// A simple enum to make sure result always succeeds.
///
/// Allow the TypeScript side to know about WatError
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub enum TransfromResult {
    Ok(InterpreterStructure),
    Err(WatError),
}

impl From<error::WatResult<InterpreterStructure>> for TransfromResult {
    fn from(value: error::WatResult<InterpreterStructure>) -> Self {
        match value {
            Ok(val) => TransfromResult::Ok(val),
            Err(err) => TransfromResult::Err(err),
        }
    }
}

/// Helper function to auto convert
#[tauri::command]
#[specta::specta]
fn transform(text: &str) -> TransfromResult {
    inner_transform(text).into()
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

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn transform_block_test() {
//         let text = "(module
//             (func $f
//                 i32.const 4
//                 (block $q
//                     i32.const 3
//                     (loop $w
//                         i32.const 2
//                         (block $e
//                             i32.const 1
//                             (if $r
//                                 (then (nop))
//                                 (else (nop))
//                             )
//                         )
//                         drop
//                     )
//                     drop
//                     (loop $t
//                         i32.const 2
//                         (block $y
//                             i32.const 1
//                             (if $u
//                                 (then (nop))
//                                 (else (nop))
//                             )
//                         )
//                         drop
//                     )
//                 )
//                 drop
//                 (block $i
//                     i32.const 2
//                     (if $o
//                         (then (nop))
//                         (else (nop))
//                     )
//                 )
//                 i32.const 0
//                 (if $p
//                     (then (i32.const 6))
//                     (else (i32.const 5))
//                 )
//                 drop
//                 nop
//             )

//         )";
//         let result = transform(text).unwrap();
//         let block = &result.func.first().unwrap().block;
//         println!(
//             "\n\nRES={}",
//             &block
//                 .root
//                 .iter()
//                 .enumerate()
//                 .map(|(i, x)| format!(
//                     "{i} => {x:?}\n\t{:?},{:?}",
//                     &block.array[x.start as usize], &block.array[x.end as usize]
//                 ))
//                 .rev()
//                 .reduce(|x, mut acc| {
//                     acc.push('\n');
//                     acc.push_str(&x);
//                     acc
//                 })
//                 .unwrap_or_default()
//         );
//     }
// }
